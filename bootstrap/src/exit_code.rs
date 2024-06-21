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
        deployment_id.clone(),
        completed,
        exit_code,
        ip,
    )).await?;
    print!("inserted target into database");

    // select all targets where deployment_id = ?1
    // if all targets are completed, update deployment state to completed
    let query = "select count(*) from target where deployment_id = ?1 and completed != true";
    let conn =  db.connect()?;  
    let mut count = conn.query(&query, params!(
        deployment_id.clone(),
    )).await?;
    let c = count.next().await?.unwrap().get::<i64>(0)?;
    if c == 0 {
        let query = "update deployment set state = completed where id = ?1";
        let conn =  db.connect()?;
        conn.execute(&query, params!(
            deployment_id.clone(),
        )).await?;

        // if promote_to_production is true, update deployment state to production
        let query = "select promote_to_production from deployment where id = ?1";
        let conn =  db.connect()?;
        let promote_to_production = conn.query(&query, params!(
            deployment_id.clone(),
        )).await?.next().await?.unwrap().get::<bool>(0)?;
        let query = "select count(*) from target where deployment_id = ?1 and completed = true and exit_code = 0";
        let mut count = conn.query(&query, params!(
            deployment_id.clone(),
        )).await?;
        let c = count.next().await?.unwrap().get::<i64>(0)?;
        // desired_count is data["min_instances"] on the deployment where data is json text in sqlite
        let query = "select data from deployment where id = ?1";
        let deployment_data = conn.query(&query, params!(
            deployment_id.clone(),
        )).await?.next().await?.unwrap().get::<String>(0)?;
        let data: serde_json::Value = serde_json::from_str(&deployment_data)?;
        let maybe_count = data.get("min_instances");
        let desired_count = match maybe_count {
            Some(count) => count.as_i64().unwrap(),
            None => return Err(anyhow::anyhow!("could not get desired count")),
        };

        let all_targets_completed =  desired_count.eq(&c);
        if promote_to_production && all_targets_completed {
            // find current production deployment and set production to false
            let template_id = conn.query("select template_id from deployment where id = ?1", params!(deployment_id.clone())).await?.next().await?.unwrap().get::<String>(0)?;
            let query = "select id from deployment where template_id = ?1 and state = production";
            let conn =  db.connect()?;
            let production_id = conn
                .query(&query, params!(template_id))
                .await?
                .next()
                .await?
                .unwrap()
                .get::<String>(0)?;
            let query = "update deployment set production = 0 where id = ?1";
            let conn =  db.connect()?;
            conn.execute(&query, params!(
                production_id,
            )).await?;

            // set current deployment to production
            let query = "update deployment set production = 1 where id = ?1";
            let conn =  db.connect()?;
            conn.execute(&query, params!(
                deployment_id.clone(),
            )).await?;
        }
    }
    Ok(())
}