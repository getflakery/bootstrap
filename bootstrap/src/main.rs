use std::env;

use libaes::Cipher;
use libsql::{params, Builder};
use serde::Serialize;

struct EC2TagData {
    turso_token: Option<String>,
    file_encryption_key: String,
    template_id: String,
    flake_url: String,
}

impl EC2TagData {
    async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let url_prefix = &config.url_prefix;

        let res = reqwest::get(&format!("{}file_encryption_key", url_prefix)).await?;
        let file_encryption_key = res.text().await?;

        let res = reqwest::get(&format!("{}template_id", url_prefix)).await?;
        let template_id = res.text().await?;

        let res = reqwest::get(&format!("{}flake_url", url_prefix)).await?;
        let flake_url = res.text().await?;
        if config.use_local {
            return Ok(Self {
                turso_token: None,
                file_encryption_key,
                template_id,
                flake_url,
            });
        }
        let res = reqwest::get(&format!("{}turso_token", url_prefix)).await?;
        let turso_token = res.text().await?;

        Ok(Self {
            turso_token: Some(turso_token),
            file_encryption_key,
            template_id,
            flake_url,
        })
    }
}

struct File {
    path: String,
    content: String,
}

#[derive(Debug, Clone)]
struct Config {
    url_prefix: String,
    sql_url: String,
    use_local: bool,
    apply_flake: bool,
}

impl Config {
    fn new() -> Self {
        let url_prefix = std::env::var("URL_PREFIX")
            .unwrap_or("http://169.254.169.254/latest/meta-data/tags/instance/".to_string())
            .to_string();
        let sql_url = std::env::var("SQL_URL")
            .unwrap_or("libsql://flakery-r33drichards.turso.io".to_string())
            .to_string();
        let use_local = std::env::var("USE_LOCAL")
            .unwrap_or("false".to_string())
            .to_string()
            == "true";
        let apply_flake = std::env::var("APPLY_FLAKE")
            .unwrap_or("true".to_string())
            .to_string()
            == "true";
        Self {
            url_prefix,
            sql_url,
            use_local,
            apply_flake,
        }
    }
}

#[derive(Serialize)]
struct LogInput {
    log: String,
}


async fn httplog(
    input: &str,
) {
    println!("{}", input);
    // just return if in test environment
    if std::env::var("TEST").unwrap_or("".to_string()) == "true" {
        return;
    }
    let log_url = std::env::var("LOG_URL").unwrap_or("http://localhost:8000/log".to_string());
    let client = reqwest::Client::new();
    let _ = client.post(&log_url).json(&LogInput { log: input.to_string() }).send().await.map_err(
        |e| {
            println!("error: {:?}", e);
        },
    );
}





#[tokio::main]
async fn main() {
    match bootstrap().await {
        Ok(_) => {
            httplog("bootstrap successful").await;
        }
        Err(e) => {
            httplog(format!("error bootstrapping: {:?}", e).as_str()).await;
        }
    }
}

async fn bootstrap() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();

    let args: Vec<String> = env::args().collect();

    if args.contains(&"--print-flake".to_string()) {
        let ec2_tag_data = EC2TagData::new(&config).await?;
        let flake_url = ec2_tag_data.flake_url;
        let res = reqwest::get(&flake_url).await?;
        let flake = res.text().await?;
        println!("{}", flake);
        return Ok(());
    }


   httplog("fetching ec2 tag data").await;

    let ec2_tag_data = EC2TagData::new(&config).await?;


    httplog("finished fetching ec2 tag data").await;
    httplog("fetching files").await;

    let sql_url = config.sql_url;

    let url = sql_url;
    let token = ec2_tag_data.turso_token;

    let mut buffer = [0; 32];
    hex::decode_to_slice(ec2_tag_data.file_encryption_key, &mut buffer)?;
    let cipher = Cipher::new_256(&buffer);
    let db = match token {
        Some(token) => Builder::new_remote(url.to_string(), token).build().await?,
        None => Builder::new_local(url).build().await?,
    };

    let conn = db.connect()?;
    let template_id = ec2_tag_data.template_id;
    let query = "SELECT f.* FROM files f JOIN template_files tf ON f.id = tf.file_id WHERE tf.template_id = ?1";
    let mut rows = conn.query(query, params!(template_id)).await?;
    let mut files: Vec<File> = Vec::new();
    while let Ok(Some(row)) = rows.next().await {
        // id,path,content,user_id,initialization_vector
        let path = row.get::<String>(1)?;
        let content = row.get::<String>(2)?;
        let initialization_vector = row.get::<String>(4)?;
        let mut iv_buffer = [0; 16];
        let content_length = content.len();
        let mut content_buffer = vec![0; content_length / 2];
        let mut cbuff = content_buffer.as_mut_slice();

        hex::decode_to_slice(initialization_vector, &mut iv_buffer)?;
        hex::decode_to_slice(content, &mut cbuff)?;
        let decrypted = cipher.cbc_decrypt(&iv_buffer, &content_buffer);
        // let dbytes = decrypted
        files.push(File {
            path,
            content: String::from_utf8(decrypted).unwrap(),
        });
    }
    httplog("finished fetching files").await;
    httplog("writing files").await;

    for file in files {
        let path_starts_with_slash = file.path.starts_with("/");
        if !path_starts_with_slash {
            let msg = format!("path not starts with slash: {}", file.path);
            httplog(&msg).await;
            return Err(msg.into());
        }
        let dirpath = std::path::Path::new(&file.path).parent().unwrap_or(std::path::Path::new("/"));
        std::fs::create_dir_all(dirpath)?;
        std::fs::write(&file.path, file.content)?;
    }
    httplog("finish writing files").await;

    Ok(())
}
