
use anyhow::Result;
use crate::EC2TagData;
use crate::File;
use reqwest::get;


async fn get_ip_address() -> Result<String> {
    let response = get("http://169.254.169.254/latest/meta-data/public-ipv4").await?;
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

pub async fn bootstrap_load_balancer(
    ec2_tag_data: &EC2TagData,
) -> Result<()> {
    println!("bootstrap_load_balancer");


    print!("writing /etc/deployment_id");
    let file = File::new(
        "/etc/deployment_id".to_string(),
        ec2_tag_data.deployment_id.clone(),
    );

    file.write()?;
    println!("wrote /etc/deployment_id");

    // try to get the public ip address of the instance
    let ip = try_get_ip_address().await?;
    println!("public ip address: {}", ip);
    Ok(())
}