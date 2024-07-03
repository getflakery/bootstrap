use anyhow::Result;

use libsql::params;

use std::fs;
use std::path::Path;

use anyhow::Context;
use libaes::Cipher;

pub struct File {
    path: String,
    content: String,
}

impl File {
    fn write(&self) -> Result<()> {
        if !self.path.starts_with('/') {
            let msg = format!("path does not start with slash: {}", self.path);
            return Err(anyhow::anyhow!(msg));
        }
        let dirpath = Path::new(&self.path).parent().unwrap_or(Path::new("/"));
        fs::create_dir_all(dirpath)?;
        fs::write(&self.path, &self.content)?;
        Ok(())
    }
}

pub async fn write_files(
    conn: libsql::Connection,
    template_id: String,
    encryption_key: String,
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
        });
    }
    // httplog("finished fetching files").await;
    println!("finished fetching files");

    println!("writing files");
    for file in files {
        file.write()?;
    }
    Ok(())
}
