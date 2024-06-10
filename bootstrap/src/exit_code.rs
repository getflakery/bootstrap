use anyhow::Result;

use libsql::params;


use reqwest::get;

async fn get_ip_address() -> Result<String> {
    let response = get("http://169.254.169.254/latest/meta-data/local-ipv4").await?;
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

pub async fn exit_code(
    exit_code: i32,
    deployment_id: String,
    db: libsql::Database,
) -> Result<()> {
    print!("rebuild exited with code {}", exit_code);

    println!("add_target");


    // try to get the public ip address of the instance
    let ip = try_get_ip_address().await?;
    println!("private ip address: {}", ip);
    
    // update target set completed = true, exit_code = ?2 where deployment_id = ?1
    let completed = true;
    let query = "update target set completed = ?2, exit_code = ?3 where deployment_id = ?1 and host = ?4";
    let conn =  db.connect()?;
    conn.execute(&query, params!(
        deployment_id,
        completed,
        exit_code,
        ip,
    )).await?;
    print!("inserted target into database");
    Ok(())
}