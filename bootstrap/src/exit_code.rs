use anyhow::Result;

use libsql::params;



pub async fn exit_code(
    exit_code: i32,
    deployment_id: String,
    db: &libsql::Database,
    ip: String
) -> Result<()> {
    print!("rebuild exited with code {}", exit_code);

    println!("add_target");


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



#[cfg(test)]
mod tests {
    use super::*;
    use libsql::{params, Builder, Database};
    use tokio;

    async fn setup_db() -> Database {
        let db = Builder::new_local(":memory:").build().await.unwrap();

        let conn = db.connect().unwrap();

        // Create the necessary tables
        conn.execute(
            "CREATE TABLE deployment (
                id TEXT PRIMARY KEY,
                state TEXT,
                promote_to_production BOOLEAN,
                data TEXT,
                template_id TEXT,
                production BOOLEAN
            )",
            params![],
        )
        .await
        .unwrap();

        conn.execute(
            "CREATE TABLE target (
                deployment_id TEXT,
                host TEXT,
                completed BOOLEAN,
                exit_code INTEGER,
                PRIMARY KEY (deployment_id, host)
            )",
            params![],
        )
        .await
        .unwrap();

        // Insert test data
        conn.execute(
            "INSERT INTO deployment (id, state, promote_to_production, data, template_id, production)
             VALUES ('test_deployment', 'in_progress', true, '{\"min_instances\": 1}', 'template_1', false)",
            params![],
        )
        .await
        .unwrap();

        conn.execute(
            "INSERT INTO target (deployment_id, host, completed, exit_code)
             VALUES ('test_deployment', 'host_1', false, NULL)",
            params![],
        )
        .await
        .unwrap();

        db
    }

    #[tokio::test]
    async fn test_exit_code() {
        let db = setup_db().await;

        let result = exit_code(0, "test_deployment".to_string(), &db, "host_1".to_string()).await;
        assert!(result.is_ok());

        let conn = db.connect().unwrap();

        // Verify target table
        let mut count = conn
            .query("SELECT count(*) FROM target WHERE deployment_id = 'test_deployment' AND completed = true", params![])
            .await
            .unwrap();
        let c = count.next().await.unwrap().unwrap().get::<i64>(0).unwrap();
        assert_eq!(c, 1);

        // Verify deployment table
        let mut count = conn
            .query("SELECT count(*) FROM deployment WHERE id = 'test_deployment' AND state = 'completed'", params![])
            .await
            .unwrap();
        let c = count.next().await.unwrap().unwrap().get::<i64>(0).unwrap();
        assert_eq!(c, 1);

        let mut count = conn
            .query("SELECT count(*) FROM deployment WHERE id = 'test_deployment' AND production = 1", params![])
            .await
            .unwrap();
        let c = count.next().await.unwrap().unwrap().get::<i64>(0).unwrap();
        assert_eq!(c, 1);
    }
}
