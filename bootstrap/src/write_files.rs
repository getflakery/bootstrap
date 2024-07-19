use anyhow::Result;

use libsql::params;
use vfs::VfsPath;

use std::path::Path;

use anyhow::Context;
use libaes::Cipher;

pub struct File {
    path: String,
    content: String,
    vfs_path: VfsPath,
}

impl File {
    fn write(&self) -> Result<()> {
        if !self.path.starts_with('/') {
            let msg = format!("path does not start with slash: {}", self.path);
            return Err(anyhow::anyhow!(msg));
        }
        let dirpath = Path::new(&self.path).parent().unwrap_or(Path::new("/"));
        let dirpath = self.vfs_path.join(dirpath.to_str().unwrap()).unwrap();
        dirpath.create_dir_all()?;
        let filepath = self.vfs_path.join(&self.path).unwrap();
        filepath.create_file()?.write_all(self.content.as_bytes())?;
        Ok(())
    }
}

pub async fn write_files(
    conn: libsql::Connection,
    template_id: String,
    encryption_key: String,
    vfspath: VfsPath,
    deployment_id: String,
) -> Result<()> {
    let query = "SELECT f.* FROM files f JOIN template_files tf ON f.id = tf.file_id WHERE tf.template_id = ?1";
    let mut rows = conn.query(query, params!(template_id)).await?;
    let mut files = Vec::new();

    let mut buffer = [0; 32];
    hex::decode_to_slice(encryption_key.clone(), &mut buffer)?;

    while let Ok(Some(row)) = rows.next().await {
        let path = row.get::<String>(1)?;
        let content = row.get::<String>(2)?;
        let initialization_vector = row.get::<String>(4)?;
        let mut iv_buffer = [0; 16];
        let content_length = content.len();
        let mut content_buffer = vec![0; content_length / 2];
        let mut cbuff = content_buffer.as_mut_slice();

        let cipher = Cipher::new_256(&buffer);
        hex::decode_to_slice(&initialization_vector, &mut iv_buffer)?;
        hex::decode_to_slice(&content, &mut cbuff)?;
        let decrypted = cipher.cbc_decrypt(&iv_buffer, &content_buffer);

        files.push(File {
            path,
            content: String::from_utf8(decrypted)
                .context("Failed to convert decrypted bytes to string")?,
            vfs_path: vfspath.clone(),
        });
    }
    // httplog("finished fetching files").await;
    println!("finished fetching files");

    println!("adding deployment id to files");

    files.push(File {
        path: "/metadata/deployment_id".to_string(),
        content: deployment_id,
        vfs_path: vfspath.clone(),
    });

    println!("finished adding deployment id to files");

    println!("writing files");
    for file in files {
        file.write()?;
    }
    Ok(())
}

// test write_files with in memory sqlite and vfs
#[cfg(test)]
mod tests {
    use super::*;
    use libsql::{params, Builder};

    use vfs::{MemoryFS, VfsPath};

    #[tokio::test]
    async fn test_write_files() {
        let vfspath: VfsPath = MemoryFS::new().into();

        let db = Builder::new_local(":memory:").build().await.unwrap();

        let conn = db.connect().unwrap();
        //         # Create tables
        // sqlite3 /tmp/db.sqlite3 "CREATE TABLE IF NOT EXISTS templates (id TEXT PRIMARY KEY);"
        // sqlite3 /tmp/db.sqlite3 "CREATE TABLE IF NOT EXISTS files (id TEXT PRIMARY KEY, path TEXT NOT NULL, content TEXT NOT NULL, user_id TEXT NOT NULL, initialization_vector TEXT NOT NULL);"
        // sqlite3 /tmp/db.sqlite3 "CREATE TABLE IF NOT EXISTS template_files (id TEXT PRIMARY KEY, file_id TEXT NOT NULL, template_id TEXT NOT NULL);"

        // sqlite3 /tmp/db.sqlite3 "CREATE TABLE IF NOT EXISTS deployments (id TEXT PRIMARY KEY, name TEXT NOT NULL, template_id TEXT NOT NULL, user_id TEXT NOT NULL, aws_instance_id TEXT, created_at INTEGER NOT NULL, host TEXT, port INTEGER, data TEXT NOT NULL, production INTEGER NOT NULL, promote_to_production INTEGER NOT NULL DEFAULT 0, state TEXT NOT NULL DEFAULT 'waiting for instances to come online');"
        // sqlite3 /tmp/db.sqlite3 "CREATE TABLE IF NOT EXISTS target (id TEXT PRIMARY KEY, deployment_id TEXT NOT NULL REFERENCES deployments(id) ON DELETE CASCADE, host TEXT NOT NULL, completed INTEGER NOT NULL DEFAULT 0, exit_code INTEGER);"

        // # Insert data
        // sqlite3 /tmp/db.sqlite3 "INSERT INTO templates (id) VALUES ('0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f');"
        // sqlite3 /tmp/db.sqlite3 "INSERT INTO files (id, path, content, user_id, initialization_vector) VALUES ('474dc715fcef9838628de248b91ad845', '/foo/bar.txt', '474dc715fcef9838628de248b91ad845', '0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f', '391827ead4c1a7fdad2dd9256d01a57a');"
        // sqlite3 /tmp/db.sqlite3 "INSERT INTO template_files (id, file_id, template_id) VALUES ('0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f', '474dc715fcef9838628de248b91ad845', '0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f');"
        // # create deployment with id 00f00f
        // sqlite3 /tmp/db.sqlite3 "INSERT INTO deployments (id, name, template_id, user_id, created_at, data, production, promote_to_production) VALUES ('00f00f', 'deployment1', '0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f', '0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f', 123456789, '{\"min_instances\": 1}', 0, 1);
        conn.execute(
            "CREATE TABLE IF NOT EXISTS templates (id TEXT PRIMARY KEY);",
            params![],
        )
        .await
        .unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (id TEXT PRIMARY KEY, path TEXT NOT NULL, content TEXT NOT NULL, user_id TEXT NOT NULL, initialization_vector TEXT NOT NULL);",
            params![],
        ).await.unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS template_files (id TEXT PRIMARY KEY, file_id TEXT NOT NULL, template_id TEXT NOT NULL);",
            params![],
        ).await.unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS deployments (id TEXT PRIMARY KEY, name TEXT NOT NULL, template_id TEXT NOT NULL, user_id TEXT NOT NULL, aws_instance_id TEXT, created_at INTEGER NOT NULL, host TEXT, port INTEGER, data TEXT NOT NULL, production INTEGER NOT NULL, promote_to_production INTEGER NOT NULL DEFAULT 0, state TEXT NOT NULL DEFAULT 'waiting for instances to come online');",
            params![],
        ).await.unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS target (id TEXT PRIMARY KEY, deployment_id TEXT NOT NULL REFERENCES deployments(id) ON DELETE CASCADE, host TEXT NOT NULL, completed INTEGER NOT NULL DEFAULT 0, exit_code INTEGER);",
            params![],
        ).await.unwrap();
        conn.execute(
            "INSERT INTO templates (id) VALUES ('0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f');",
            params![],
        ).await.unwrap();
        conn.execute(
            "INSERT INTO files (id, path, content, user_id, initialization_vector) VALUES ('474dc715fcef9838628de248b91ad845', '/foo/bar.txt', '474dc715fcef9838628de248b91ad845', '0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f', '391827ead4c1a7fdad2dd9256d01a57a');",
            params![],
        ).await.unwrap();
        conn.execute(
            "INSERT INTO template_files (id, file_id, template_id) VALUES ('0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f', '474dc715fcef9838628de248b91ad845', '0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f');",
            params![],
        ).await.unwrap();
        conn.execute(
            "INSERT INTO deployments (id, name, template_id, user_id, created_at, data, production, promote_to_production) VALUES ('00f00f', 'deployment1', '0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f', '0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f', 123456789, '{\"min_instances\": 1}', 0, 1);",
            params![],
        ).await.unwrap();

        write_files(
            conn,
            "0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f".to_string(),
            "0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f".to_string(),
            vfspath.clone(),
            "deployment1".to_string(),
        )
        .await
        .unwrap();
        // assert file was written
        let file = vfspath.join("/foo/bar.txt").unwrap();
        assert!(file.exists().unwrap());
        let content = file.read_to_string().unwrap();
        assert_eq!(content, "secret");
        // assert /metadata/deployment_id was written
        let file = vfspath.join("/metadata/deployment_id").unwrap();
        assert!(file.exists().unwrap());
        let content = file.read_to_string().unwrap();
        assert_eq!(content, "deployment1");
    }
}
