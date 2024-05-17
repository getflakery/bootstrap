use std::env;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use libaes::Cipher;
use libsql::{params, Builder};
use serde::Serialize;
use std::process::ExitCode;
use flakery_client::types::{CreateListenerInput, Mapping};

use reqwest::header;

struct EC2TagData {
    turso_token: Option<String>,
    file_encryption_key: String,
    template_id: String,
    flake_url: String,
    deployment_id: String,
}

impl EC2TagData {
    async fn new(config: &Config) -> Result<Self> {
        let url_prefix = &config.url_prefix;

        let file_encryption_key = reqwest::get(&format!("{}file_encryption_key", url_prefix)).await?.text().await?;
        let template_id = reqwest::get(&format!("{}template_id", url_prefix)).await?.text().await?;
        let flake_url = reqwest::get(&format!("{}flake_url", url_prefix)).await?.text().await?;
        let deployment_id = reqwest::get(&format!("{}deployment_id", url_prefix)).await?.text().await?;

        if config.use_local {
            return Ok(Self {
                turso_token: None,
                file_encryption_key,
                template_id,
                flake_url,
                deployment_id,
            });
        }

        let turso_token = reqwest::get(&format!("{}turso_token", url_prefix)).await?.text().await?;

        Ok(Self {
            turso_token: Some(turso_token),
            file_encryption_key,
            template_id,
            flake_url,
            deployment_id,
        })
    }
}

struct File {
    path: String,
    content: String,
}

#[derive(Debug)]
struct Config {
    url_prefix: String,
    sql_url: String,
    use_local: bool,
    apply_flake: bool,
    set_debug_header: bool,
    rclient: reqwest::Client,
}

impl Config {
    fn new() -> Self {
        let url_prefix = std::env::var("URL_PREFIX").unwrap_or("http://169.254.169.254/latest/meta-data/tags/instance/".to_string());
        let sql_url = std::env::var("SQL_URL").unwrap_or("libsql://flakery-r33drichards.turso.io".to_string());
        let use_local = std::env::var("USE_LOCAL").unwrap_or("false".to_string()) == "true";
        let apply_flake = std::env::var("APPLY_FLAKE").unwrap_or("true".to_string()) == "true";
        let set_debug_header = std::env::var("SET_DEBUG_HEADER").unwrap_or("false".to_string()) == "true";
        
        let rclient = {
            let dur = std::time::Duration::from_secs(15);
            let mut builder = reqwest::ClientBuilder::new().connect_timeout(dur).timeout(dur);
            
            let mut headers: header::HeaderMap = header::HeaderMap::new();
            if set_debug_header {
                headers.insert("Debug", header::HeaderValue::from_static("true"));
            }
            builder = builder.default_headers(headers);
            builder.build().unwrap()
        };

        Self {
            url_prefix,
            sql_url,
            use_local,
            apply_flake,
            set_debug_header,
            rclient,
        }
    }
}

#[derive(Serialize)]
struct LogInput {
    log: String,
}

async fn httplog(input: &str) {
    println!("{}", input);
    if std::env::var("TEST").unwrap_or("".to_string()) == "true" {
        return;
    }
    let log_url = std::env::var("LOG_URL").unwrap_or("http://localhost:8000/log".to_string());
    let client = reqwest::Client::new();
    let _ = client.post(&log_url)
        .json(&LogInput { log: input.to_string() })
        .send()
        .await
        .map_err(|e| println!("error: {:?}", e));
}

#[tokio::main]
async fn main() -> ExitCode {
    match bootstrap().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            httplog(&format!("error bootstrapping: {:?}", e)).await;
            return ExitCode::from(42);
        }
    }
}

async fn bootstrap() -> Result<()> {
    let config = Config::new();

    let args: Vec<String> = env::args().collect();

    if args.contains(&"--print-flake".to_string()) {
        let ec2_tag_data = EC2TagData::new(&config).await?;
        println!("{}", ec2_tag_data.flake_url);
        return Ok(());
    }

    if args.contains(&"--print-deployment-id".to_string()) {
        let ec2_tag_data = EC2TagData::new(&config).await?;
        println!("{}", ec2_tag_data.deployment_id);
        return Ok(());
    }

    if args.contains(&"--attach-lb".to_string()) {
        let ec2_tag_data = EC2TagData::new(&config).await?;
        let deployment_id = ec2_tag_data.deployment_id;

        flakery_client::Client::new_with_client("http://localhost:8000", config.rclient)
            .handlers_create_listener_create_listener(&CreateListenerInput {
                deployment_id: deployment_id.clone(),
                mappings: vec![
                    Mapping {
                        listener_port: todo!("443"),
                        target_port: todo!("8000"),
                    },
                ],
            })
            .await?;

        return Ok(());
    }

    httplog("fetching ec2 tag data").await;
    let ec2_tag_data = EC2TagData::new(&config).await?;
    httplog("finished fetching ec2 tag data").await;

    httplog("fetching files").await;
    let sql_url = config.sql_url;
    let token = ec2_tag_data.turso_token;
    let mut buffer = [0; 32];
    hex::decode_to_slice(&ec2_tag_data.file_encryption_key, &mut buffer)?;
    let cipher = Cipher::new_256(&buffer);
    let db = match token {
        Some(token) => Builder::new_remote(sql_url.to_string(), token).build().await?,
        None => Builder::new_local(sql_url).build().await?,
    };

    let conn = db.connect()?;
    let query = "SELECT f.* FROM files f JOIN template_files tf ON f.id = tf.file_id WHERE tf.template_id = ?1";
    let mut rows = conn.query(query, params!(ec2_tag_data.template_id)).await?;
    let mut files = Vec::new();
    while let Ok(Some(row)) = rows.next().await {
        let path = row.get::<String>(1)?;
        let content = row.get::<String>(2)?;
        let initialization_vector = row.get::<String>(4)?;
        let mut iv_buffer = [0; 16];
        let content_length = content.len();
        let mut content_buffer = vec![0; content_length / 2];
        let mut cbuff = content_buffer.as_mut_slice();

        hex::decode_to_slice(&initialization_vector, &mut iv_buffer)?;
        hex::decode_to_slice(&content, &mut cbuff)?;
        let decrypted = cipher.cbc_decrypt(&iv_buffer, &content_buffer);

        files.push(File {
            path,
            content: String::from_utf8(decrypted).context("Failed to convert decrypted bytes to string")?,
        });
    }
    httplog("finished fetching files").await;

    httplog("writing files").await;
    for file in files {
        if !file.path.starts_with('/') {
            let msg = format!("path does not start with slash: {}", file.path);
            httplog(&msg).await;
            return Err(anyhow::anyhow!(msg));
        }
        let dirpath = Path::new(&file.path).parent().unwrap_or(Path::new("/"));
        fs::create_dir_all(dirpath)?;
        fs::write(&file.path, &file.content)?;
    }
    httplog("finished writing files").await;
    httplog("bootstrap successful").await;

    Ok(())
}
