use crate::EC2TagData;
use anyhow::Result;

use libsql::params;
use reqwest::get;

async fn get_ip_address() -> Result<String> {
    let response = get("http://169.254.169.254/latest/meta-data/private-ipv4").await?;
    let ip_address = response.text().await?;
    Ok(ip_address)
}

async fn try_get_ip_address() -> Result<String> {
    for _ in 0..100 {
        match get_ip_address().await {
            Ok(ip_address) => return Ok(ip_address),
            Err(e) => {
                eprintln!("{:?} {}:{}", e, file!(), line!());
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
    Err(anyhow::anyhow!("could not get ip address"))
}

pub async fn add_target(
    ec2_tag_data: &EC2TagData,
    db: libsql::Database,
) -> Result<()> {
    println!("add_target");


    // try to get the public ip address of the instance
    let ip = try_get_ip_address().await?;
    println!("private ip address: {}", ip);

    let id = uuid::Uuid::new_v4();
    let deployment_id = ec2_tag_data.deployment_id.clone();

    let query = "INSERT INTO targets (id, deployment_id, host) VALUES ($1, $2, $3)";
    let conn =  db.connect()?;
    conn.execute(&query, params!(
        id.to_string(),
        deployment_id,
        ip,
    )).await?;
    Ok(())
}
