use anyhow::Result;


use std::env;
use std::error::Error;
use std::fmt;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Response {
    host: String,
    publickey: String,
}

#[derive(Debug)]
struct MissingTokenError;

impl fmt::Display for MissingTokenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "USER_TOKEN is not set")
    }
}

impl Error for MissingTokenError {}

async fn get_substituters() -> Result<Vec<String>, Box<dyn Error>> {
    let user_token = env::var("USER_TOKEN")?;

    if user_token.is_empty() {
        return Err(Box::new(MissingTokenError));
    }

    let url = "https://flakery.dev/api/v0/user/private-binary-cache/host";
    let client = Client::new();

    let resp = client.get(url)
        .header("Authorization", format!("Bearer {}", user_token))
        .header("Content-Type", "application/json")
        .send().await?;

    if !resp.status().is_success() {
        return Err(format!("GET request failed with status: {}", resp.status()).into());
    }

    let r: Response = resp.json().await?;

    Ok(vec![
        "--option".to_string(),
        "trusted-public-keys".to_string(),
        r.publickey,
        "--option".to_string(),
        "substituters".to_string(),
        format!("http://{}:5000", r.host),
    ])
}


pub async fn print_extra_options() {
    if let Ok(subs) = get_substituters().await {
        // join strings with spaces 
        let msg = subs.join(" ");
        print!("{}", msg)
    } 
}