use libaes::Cipher;
use libsql::{params, Builder};

// file struct
struct EC2TagData {
    turso_token: String,
    file_encryption_key: String,
    template_id: String,
    flake_url: String,
}

impl EC2TagData {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let res = reqwest::get("http://169.254.169.254/latest/meta-data/tags/instance/turso_token")
            .await?;
        let turso_token = res.text().await?;

        let res = reqwest::get("http://169.254.169.254/latest/meta-data/tags/instance/file_encryption_key").await?;
        let file_encryption_key = res.text().await?;

        let res = reqwest::get("http://169.254.169.254/latest/meta-data/tags/instance/template_id").await?;
        let template_id = res.text().await?;

        let res = reqwest::get("http://169.254.169.254/latest/meta-data/tags/instance/flake_url").await?;
        let flake_url = res.text().await?;

        Ok(Self {
            turso_token,
            file_encryption_key,
            template_id,
            flake_url
        })
    }
}

struct File {
    path: String,
    content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let ec2_tag_data = EC2TagData::new().await?;

    let url = "libsql://flakery-r33drichards.turso.io".to_string();
    let token = ec2_tag_data.turso_token;

    let mut buffer = [0; 32];
    hex::decode_to_slice(
        ec2_tag_data.file_encryption_key,
        &mut buffer,
    )
    .unwrap();
    let cipher = Cipher::new_256(&buffer);

    let db = Builder::new_remote(url, token).build().await?;
    let conn = db.connect().unwrap();
    let template_id = ec2_tag_data.template_id;
    let query = "SELECT f.* FROM files f JOIN template_files tf ON f.id = tf.file_id WHERE tf.template_id = ?1";
    let mut rows = conn.query(query, params!(template_id)).await?;
    let mut files: Vec<File> = Vec::new();
    while let Ok(Some(row)) = rows.next().await {
        // id,path,content,user_id,initialization_vector
        let id = row.get::<String>(0).unwrap();
        let path = row.get::<String>(1).unwrap();
        let content = row.get::<String>(2).unwrap();
        let initialization_vector = row.get::<String>(4).unwrap();
        println!(
            "id: {}, path: {}, content: {},  initialization_vector: {}",
            id, path, content, initialization_vector
        );
        let mut iv_buffer = [0; 16];
        let content_length = content.len();
        let mut content_buffer = vec![0; content_length / 2];
        let mut cbuff = content_buffer.as_mut_slice();

        hex::decode_to_slice(initialization_vector, &mut iv_buffer).unwrap();
        hex::decode_to_slice(content, &mut cbuff).unwrap();
        let decrypted = cipher.cbc_decrypt(&iv_buffer, &content_buffer);
        // let dbytes = decrypted
        files.push(File {
            path,
            content: String::from_utf8(decrypted).unwrap(),
        });
    }

    for file in files {
        let dirpath = std::path::Path::new(&file.path).parent().unwrap();
        std::fs::create_dir_all(dirpath)?;
        std::fs::write(&file.path, file.content)?;
    }

    // apply flake with exec
    // 	err = session.Run(fmt.Sprintf("nixos-rebuild switch --impure --flake '%s'", flake_url))
    let output = tokio::process::Command::new("nixos-rebuild")
        .arg("switch")
        .arg("--impure")
        .arg("--flake")
        .arg(ec2_tag_data.flake_url)
        .output()
        .await?;
    println!("status: {}", output.status);
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}
