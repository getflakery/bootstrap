use libaes::Cipher;
use libsql::{params, Builder};


// file struct
struct File {
    key: String,
    value: String,
}

struct Deployment {
    id: String,
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "libsql://flakery-r33drichards.turso.io".to_string();
    let token ="".to_string();

    // let url = env::var("LIBSQL_URL").expect("LIBSQL_URL must be set");
    // let token = env::var("LIBSQL_AUTH_TOKEN").unwrap_or_default();


     let mut buffer = [0; 32];
     hex::decode_to_slice("", &mut buffer).unwrap();
    let cipher = Cipher::new_256(&buffer);

    let db = Builder::new_remote(url, token).build().await?;
    let conn = db.connect().unwrap();
    let mut rows = conn.query("SELECT * FROM files", params!()).await?;

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
        let mut ivBuffer = [0; 16];
        let contentLength = content.len();
        let mut contentBuffer = vec![0; contentLength/2];
        let mut cbuff = contentBuffer.as_mut_slice();



        hex::decode_to_slice(initialization_vector, &mut ivBuffer).unwrap();
        hex::decode_to_slice(content, &mut cbuff).unwrap();
        let decrypted = cipher.cbc_decrypt(&ivBuffer, &contentBuffer);
        // let dbytes = decrypted
        println!("decrypted: {:?}", String::from_utf8(decrypted).unwrap());


    }

    // let res = reqwest::get("http://169.254.169.254/latest/meta-data/tags/instance").await?;
    // let body = res.text().await?;

    // let fileKeys = body.split("\n").filter(
    //     |key| key.starts_with("file:")
    // ).collect::<Vec<&str>>();

    // let files = fileKeys.iter().map(|key| {
    //     let res = reqwest::get(
    //         format!("http://169.254.169.254/latest/meta-data/tags/instance/{}", key)
    //     ).await?;
    //     let value = res.text().await?;
    //     File {
    //         key: key.to_string().strip_prefix("file:").unwrap_or(&key).to_string(),
    //         value: value,
    //     }
    // }).collect::<Vec<File>>();

    // for file in files {
    //     println!("{}: {}", file.key, file.value);
    // }

    Ok(())
}
