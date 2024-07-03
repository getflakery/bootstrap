use aws_config::default_provider::token;
use std::env;
use vfs::{PhysicalFS, VfsPath};

use anyhow::Result;
use libsql::Builder;
use std::process::ExitCode;

mod add_target;
use add_target::add_target;

mod wrap_with_deployment_id;
use wrap_with_deployment_id::wrap_with_deployment_id;

mod exit_code;
use exit_code::exit_code;

mod write_files;
use write_files::write_files;

use reqwest::get;

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

        let file_encryption_key = reqwest::get(&format!("{}file_encryption_key", url_prefix))
            .await?
            .text()
            .await?;
        let template_id = reqwest::get(&format!("{}template_id", url_prefix))
            .await?
            .text()
            .await?;
        let flake_url = reqwest::get(&format!("{}flake_url", url_prefix))
            .await?
            .text()
            .await?;
        let deployment_id = reqwest::get(&format!("{}deployment_id", url_prefix))
            .await?
            .text()
            .await?;
        let github_token = reqwest::get(&format!("{}github_token", url_prefix))
            .await?
            .text()
            .await?;
        let bootstrap_args = reqwest::get(&format!("{}bootstrap_args", url_prefix))
            .await?
            .text()
            .await?
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
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

        let turso_token = reqwest::get(&format!("{}turso_token", url_prefix))
            .await?
            .text()
            .await?;

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

#[derive(Debug, Clone)]
pub struct Config {
    url_prefix: String,
    ip_v4_url_prefix: String,
    sql_url: String,
    use_local: bool,
}

impl Config {
    fn new() -> Self {
        let url_prefix = std::env::var("URL_PREFIX")
            .unwrap_or("http://169.254.169.254/latest/meta-data/tags/instance/".to_string());
        let ip_v4_url_prefix = std::env::var("IP_V4_URL_PREFIX")
            .unwrap_or("http://169.254.169.254/latest/meta-data/".to_string());
        let sql_url = std::env::var("SQL_URL")
            .unwrap_or("libsql://flakery-r33drichards.turso.io".to_string());
        let use_local = std::env::var("USE_LOCAL").unwrap_or("false".to_string()) == "true";
        Self {
            url_prefix,
            ip_v4_url_prefix,
            sql_url,
            use_local,
        }
    }

    async fn get_ip_address(self) -> Result<String> {
        let response = get(format!("{}local-ipv4", self.ip_v4_url_prefix)).await?;
        let ip_address = response.text().await?;
        Ok(ip_address)
    }

    async fn try_get_ip_address(self) -> Result<String> {
        for _ in 0..100 {
            match self.clone().get_ip_address().await {
                Ok(ip_address) => return Ok(ip_address),
                Err(e) => {
                    eprintln!("{:?} {}:{}", e, file!(), line!());
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }
        }
        Err(anyhow::anyhow!("could not get ip address"))
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

// get args value after arg
fn arg_value(args: Vec<String>, arg: String) -> Result<String> {
    let index = args.iter().position(|a| *a == arg).unwrap() + 1;
    let value = args[index].parse::<String>();
    match value {
        Ok(value) => Ok(value),
        Err(_) => Err(anyhow::anyhow!("could not parse value")),
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
    let root: VfsPath = PhysicalFS::new("/").into();

    // if args contains --write-files, write files and return
    if args.contains(&"--write-files".to_string()) {
        
        let sql_url = config.clone().sql_url;
        let db: libsql::Database;
        if args.contains(&"--turso-token".to_string()) {
            let token = arg_value(args.clone(), "--turso-token".to_string())?;
            db = Builder::new_remote(sql_url.to_string(), token)
                .build()
                .await?
        } else {
            db = Builder::new_local(sql_url).build().await?
        };

        let conn: libsql::Connection = db.connect()?;
        let template_id = arg_value(args.clone(), "--template-id".to_string())?;
        let encryption_key = arg_value(args.clone(), "--encryption-key".to_string())?;
        write_files(conn, template_id, encryption_key, root).await?;
        return Ok(());
    }

    println!("fetching ec2 tag data");
    let mut ec2_tag_data = EC2TagData::new(&config).await?;
    println!("fetched ec2 tag data");

    args.append(&mut ec2_tag_data.bootstrap_args);

    let sql_url = config.clone().sql_url;
    let token = ec2_tag_data.clone().turso_token;
    let db: libsql::Database = match token {
        Some(token) => {
            Builder::new_remote(sql_url.to_string(), token)
                .build()
                .await?
        }
        None => Builder::new_local(sql_url).build().await?,
    };

    if args.contains(&"--exit-code".to_string()) {
        // get arg after --exit-code
        let ecode =
            args[args.iter().position(|arg| arg == "--exit-code").unwrap() + 1].parse::<i32>();
        if let Ok(ecode) = ecode {
            let ip = config.clone().try_get_ip_address().await?;
            let conn: libsql::Connection = db.connect().unwrap();

            return exit_code(ecode, ec2_tag_data.deployment_id, &conn, ip).await;
        }
        return Err(anyhow::anyhow!("could not parse exit code"));
    }

    println!("connecting to db");
    let conn: libsql::Connection = db.connect()?;
    println!("connected to db");

    println!("fetching files");
    println!("querying files");
    let template_id = ec2_tag_data.clone().template_id;
    let encryption_key = ec2_tag_data.clone().file_encryption_key;
    write_files(conn, template_id, encryption_key, root).await?;
    println!("finished writing files");
    println!("finished bootstrapping");

    println!("adding target");
    let ip = config.clone().try_get_ip_address().await?;
    add_target(&ec2_tag_data, db, ip).await?; // Pass the cloned sql_url to add_target
    println!("added target");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_value() {
        let args = vec![
            "bootstrap".to_string(),
            "--turso-token".to_string(),
            "token".to_string(),
        ];
        let arg = "--turso-token".to_string();
        let value = arg_value(args, arg).unwrap();
        assert_eq!(value, "token".to_string());
    }
}
