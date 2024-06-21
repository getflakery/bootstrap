use std::env;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use libaes::Cipher;
use libsql::{params, Builder};
use std::process::ExitCode;

mod add_target;
use add_target::add_target;

mod wrap_with_deployment_id;
use wrap_with_deployment_id::wrap_with_deployment_id;

mod exit_code;
use exit_code::exit_code;

#[derive(Clone, Debug)]
pub struct EC2TagData {
    turso_token: Option<String>,
    file_encryption_key: String,
    template_id: String,
    flake_url: String,
    deployment_id: String,
    github_token: String,
    bootstrap_args: Vec<String>,
}

impl EC2TagData {
    async fn new(config: &Config) -> Result<Self> {
        let url_prefix = &config.url_prefix;

        let file_encryption_key = reqwest::get(&format!("{}file_encryption_key", url_prefix)).await?.text().await?;
        let template_id = reqwest::get(&format!("{}template_id", url_prefix)).await?.text().await?;
        let flake_url = reqwest::get(&format!("{}flake_url", url_prefix)).await?.text().await?;
        let deployment_id = reqwest::get(&format!("{}deployment_id", url_prefix)).await?.text().await?;
        let github_token = reqwest::get(&format!("{}github_token", url_prefix)).await?.text().await?;
        let bootstrap_args = reqwest::get(&format!("{}bootstrap_args", url_prefix)).await?.text().await?.split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>();
        if config.use_local {
            return Ok(Self {
                turso_token: None,
                file_encryption_key,
                template_id,
                flake_url,
                deployment_id,
                github_token,
                bootstrap_args,
            });
        }

        let turso_token = reqwest::get(&format!("{}turso_token", url_prefix)).await?.text().await?;

        Ok(Self {
            turso_token: Some(turso_token),
            file_encryption_key,
            template_id,
            flake_url,
            deployment_id,
            github_token,
            bootstrap_args,
        })
    }
}

pub struct File {
    path: String,
    content: String,
}

impl File {


    fn write(&self) -> Result<()> {
        if !self.path.starts_with('/') {
            let msg = format!("path does not start with slash: {}", self.path);
            return Err(anyhow::anyhow!(msg));
        }
        let dirpath = Path::new(&self.path).parent().unwrap_or(Path::new("/"));
        fs::create_dir_all(dirpath)?;
        fs::write(&self.path, &self.content)?;
        Ok(())
    }

}

#[derive(Debug, Clone)]
pub struct Config {
    url_prefix: String,
    ip_v4_url_prefix: String,
    sql_url: String,
    use_local: bool,
}

impl Config {
    fn new() -> Self {
        let url_prefix = std::env::var("URL_PREFIX").unwrap_or("http://169.254.169.254/latest/meta-data/tags/instance/".to_string());
        let ip_v4_url_prefix = std::env::var("IP_V4_URL_PREFIX").unwrap_or("http://169.254.169.254/latest/meta-data/".to_string());
        let sql_url = std::env::var("SQL_URL").unwrap_or("libsql://flakery-r33drichards.turso.io".to_string());
        let use_local = std::env::var("USE_LOCAL").unwrap_or("false".to_string()) == "true";
        Self {
            url_prefix,
            ip_v4_url_prefix,
            sql_url,
            use_local,
        }
    }
}




#[tokio::main]
async fn main() -> ExitCode {
    match bootstrap().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{:?} {}:{}", e, file!(), line!());
            return ExitCode::from(42);
        }
    }
}

async fn bootstrap() -> Result<()> {
    let mut args: Vec<String> = env::args().collect();


    if args.contains(&"--debug-error".to_string()) {
        return Err(anyhow::anyhow!("debug error"));
    }
    
    let config = Config::new();

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

    if args.contains(&"--wrap_with_deployment_id".to_string()) {
        let ec2_tag_data = EC2TagData::new(&config).await?;
        wrap_with_deployment_id(&ec2_tag_data.deployment_id);
        // wrap_with_deployment_id("deployment_id");
        return Ok(());
    }

    if args.contains(&"--print-github-token".to_string()) {
        let ec2_tag_data = EC2TagData::new(&config).await?;
        println!("{}", ec2_tag_data.github_token);
        return Ok(());
    }


    println!("fetching ec2 tag data");
    let mut ec2_tag_data = EC2TagData::new(&config).await?;
    println!("fetched ec2 tag data");


    args.append(&mut ec2_tag_data.bootstrap_args);


    let sql_url = config.sql_url;
    let token = ec2_tag_data.clone().turso_token;
    let mut buffer = [0; 32];
    hex::decode_to_slice(&ec2_tag_data.file_encryption_key, &mut buffer)?;
    let cipher = Cipher::new_256(&buffer);
    let db: libsql::Database = match token {
        Some(token) => Builder::new_remote(sql_url.to_string(), token).build().await?,
        None => Builder::new_local(sql_url).build().await?,
    };

    if args.contains(&"--exit-code".to_string()) {
        // get arg after --exit-code
        let ecode = args[args.iter().position(|arg| arg == "--exit-code").unwrap() + 1].parse::<i32>();
        if let Ok(ecode) = ecode {
            return exit_code(ecode, ec2_tag_data.deployment_id, db).await;
        }
        return Err(anyhow::anyhow!("could not parse exit code"));
    }

    println!("connecting to db");
    let conn = db.connect()?;
    println!("connected to db");


    println!("fetching files");
    println!("querying files");
    let query = "SELECT f.* FROM files f JOIN template_files tf ON f.id = tf.file_id WHERE tf.template_id = ?1";
    let mut rows = conn.query(query, params!(ec2_tag_data.clone().template_id)).await?;
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
    // httplog("finished fetching files").await;
    println!("finished fetching files");

    println!("writing files");
    for file in files {
        file.write()?;
    }
    println!("finished writing files");
    println!("finished bootstrapping"); 

    println!("adding target");
    add_target(&ec2_tag_data, db, config.clone()).await?;
    println!("added target");

    Ok(())
}
