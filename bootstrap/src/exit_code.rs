use anyhow::Result;

use libsql::params;


pub async fn exit_code(
    exit_code: i32,
    deployment_id: String,
    db: libsql::Database,
) -> Result<()> {
    print!("rebuild exited with code {}", exit_code);
    
    // update target set completed = true, exit_code = ?2 where deployment_id = ?1
    let completed = true;
    let query = "update target set completed = ?2, exit_code = ?3 where id = ?1";
    let conn =  db.connect()?;
    conn.execute(&query, params!(
        deployment_id,
        completed,
        exit_code,
    )).await?;
    print!("inserted target into database");
    Ok(())
}