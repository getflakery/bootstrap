
use crate::EC2TagData;
use anyhow::Result;

use libsql::params;

pub async fn add_target(
    ec2_tag_data: &EC2TagData,
    db: libsql::Database,
    ip : String,
) -> Result<()> {
    println!("add_target");

    let id = uuid::Uuid::new_v4();
    let deployment_id = ec2_tag_data.deployment_id.clone();

    // this def qualifies as tech debt 
    println!("inserting target into database");
    println!("id: {}", id);
    println!("deployment_id: {}", deployment_id);
    println!("ip: {}", ip);
    let query = "INSERT INTO target (id, deployment_id, host) VALUES (?1, ?2, ?3)";
    let conn =  db.connect()?;
    conn.execute(&query, params!(
        id.to_string(),
        deployment_id,
        ip,
    )).await?;
    print!("inserted target into database");
    Ok(())
}