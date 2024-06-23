use anyhow::Result;

use libsql::params;
pub async fn exit_code(
    exit_code: i32,
    deployment_id: String,
    conn: &libsql::Connection,
    ip: String,
) -> Result<()> {
    println!("rebuild exited with code {}", exit_code);

    println!("set target completed to true and exit code to {}", exit_code);

    let out = conn.execute(
        "UPDATE target SET completed = ?1, exit_code = ?2 where deployment_id = ?3 and host = ?4",
        params![
            1i32,
            exit_code,
            deployment_id.clone(),
            ip.clone(),
        ],
    ).await;
    match out {
        Ok(u) => {
            println!("updated {} rows", u);
            if u == 0 {
                return Err(anyhow::anyhow!("no rows updated"));
            }
        }
        Err(e) => {
            println!("error inserting target into database: {:?}", e);
            return Err(anyhow::anyhow!("error inserting target into database"));
        }
    }



    // select all targets where deployment_id = ?1
    // if all targets are completed, update deployment state to completed
    let query = "select count(*) from target where deployment_id = ?1 and completed != true";
    let mut count = conn.query(&query, params!(deployment_id.clone(),)).await?;
    let c = count.next().await?.unwrap().get::<i64>(0)?;
    if c == 0 {

        // if promote_to_production is true, update deployment state to production
        let query = "select promote_to_production from deployments where id = ?1";
        let promote_to_production = conn
            .query(&query, params!(deployment_id.clone(),))
            .await?
            .next()
            .await?
            .unwrap()
            .get::<bool>(0)?;
        println!("promote_to_production: {}", promote_to_production);
        let query = "select count(*) from target where deployment_id = ?1 and completed = 1 and exit_code = 0";
        let mut count = conn.query(&query, params!(deployment_id.clone(),)).await?;
        
        let c = count.next().await?.unwrap().get::<i64>(0)?;
        println!("c: {}", c);
        // desired_count is data["min_instances"] on the deployment where data is json text in sqlite
        let query = "select data from deployments where id = ?1";
        let deployment_data = conn
            .query(&query, params!(deployment_id.clone(),))
            .await?
            .next()
            .await?
            .unwrap()
            .get::<String>(0)?;
        let data: serde_json::Value = serde_json::from_str(&deployment_data)?;
        let maybe_count = data.get("min_instances");
        let desired_count = match maybe_count {
            Some(count) => count.as_i64().unwrap(),
            None => return Err(anyhow::anyhow!("could not get desired count")),
        };

        println!("desired_count: {}", desired_count);
        let all_targets_completed = desired_count.eq(&c);
        println!("all_targets_completed: {}", all_targets_completed);
        if promote_to_production && all_targets_completed {
            // find current production deployment and set production to false
            let template_id = conn
                .query(
                    "select template_id from deployments where id = ?1",
                    params!(deployment_id.clone()),
                )
                .await?
                .next()
                .await?
                .unwrap()
                .get::<String>(0)?;
            let query = "select id from deployments where template_id = ?1 and production = 1";
            let production_id = conn
                .query(&query, params!(template_id))
                .await?
                .next()
                .await;
            if let Ok(Some(production_id)) = production_id {
                // todo this is not tested yet
                let production_id = production_id.get::<String>(0)?;
                let query = "update deployments set production = 0 where id = ?1";
                conn.execute(&query, params!(production_id,)).await?;
            }

            // set current deployment to production
            let query = "update deployments set production = 1 where id = ?1";
            let out = conn.execute(&query, params!(deployment_id.clone(),))
                .await;
            match out {
                Ok(u) => {
                    println!("updated {} rows", u);
                    if u == 0 {
                        return Err(anyhow::anyhow!("no rows updated"));
                    }
                }
                Err(e) => {
                    println!("error inserting target into database: {:?}", e);
                    return Err(anyhow::anyhow!("error inserting target into database"));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use libsql::{params, Builder};
    use tokio;
    use std::fs::remove_file;


    #[tokio::test]
    async fn test_exit_code() {
        // rm /tmp/db.test.sqlite
        remove_file("/tmp/db.test.sqlite").unwrap_or_default();
        let db = Builder::new_local("/tmp/db.test.sqlite").build().await.unwrap();

        let conn = db.connect().unwrap();

        // Create the necessary tables
        conn.execute(
            "CREATE TABLE deployments (
                id TEXT PRIMARY KEY,
                state TEXT,
                promote_to_production INTEGER,
                data TEXT,
                template_id TEXT,
                production INTEGER
            )",
            params![],
        )
        .await
        .unwrap();

        conn.execute(
            "CREATE TABLE target (
                deployment_id TEXT,
                host TEXT,
                completed INTEGER,
                exit_code INTEGER,
                PRIMARY KEY (deployment_id, host)
            )",
            params![],
        )
        .await
        .unwrap();

        // Insert test data
        conn.execute(
            "INSERT INTO deployments (id, state, promote_to_production, data, template_id, production)
             VALUES ('test_deployment', 'in_progress', true, '{\"min_instances\": 1}', 'template_1', false)",
            params![],
        )
        .await
        .unwrap();

        conn.execute(
            "INSERT INTO target (deployment_id, host, completed, exit_code)
             VALUES ('test_deployment', 'host_1', 0, NULL)",
            params![],
        )
        .await
        .unwrap();

        // print the tables
        let mut rows = conn
            .query("SELECT * FROM deployments", params![])
            .await
            .unwrap();
        while let Ok(Some(row)) = rows.next().await {
            println!("{:?}", row);
        }

        let mut rows = conn.query("SELECT completed FROM target", params![]).await.unwrap();
        while let Ok(Some(row)) = rows.next().await {
            println!("{:?}", row);
        }

        let result = exit_code(0, "test_deployment".to_string(), &conn, "host_1".to_string()).await;
        println!("{:?}", result);
        assert!(result.is_ok());


        // Verify target table
        let mut count = conn
            .query("SELECT count(*) FROM target WHERE deployment_id = 'test_deployment' AND completed = true", params![])
            .await
            .unwrap();
        let c = count.next().await.unwrap().unwrap().get::<i64>(0).unwrap();
        assert_eq!(c, 1);

        // Verify deployment table
        let mut count = conn
            .query("SELECT count(*) FROM deployments WHERE id = 'test_deployment' AND production = 1", params![])
            .await
            .unwrap();
        let c = count.next().await.unwrap().unwrap().get::<i64>(0).unwrap();
        assert_eq!(c, 1);
    }
}
