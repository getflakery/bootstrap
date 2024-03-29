
// file struct
struct File {
    key: String,
    value: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let res = reqwest::get("http://169.254.169.254/latest/meta-data/tags/instance").await?;
    let body = res.text().await?;

    let fileKeys = body.split("\n").filter(
        |key| key.starts_with("file:")
    ).collect::<Vec<&str>>();

    let files = fileKeys.iter().map(|key| {
        let res = reqwest::get(
            format!("http://169.254.169.254/latest/meta-data/tags/instance/{}", key)
        ).await?;
        let value = res.text().await?;
        File {
            key: key.to_string().strip_prefix("file:").unwrap_or(&key).to_string(),
            value: value,
        }
    }).collect::<Vec<File>>();

    for file in files {
        println!("{}: {}", file.key, file.value);
    }





    Ok(())
}
