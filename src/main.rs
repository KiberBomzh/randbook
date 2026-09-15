use std::io::Write;
use std::path::{Path, PathBuf};

use reqwest::Client;


const URL_PREFFIX: &str = "https://";
const URL: &str = "searchfloor.org";

// 14.09.2026
const MAX_INDEX: u32 = 27118;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = URL_PREFFIX.to_string() + URL;
    let client = reqwest::Client::new();

    let amount = 10;
    let dir = PathBuf::from(".");

    for _ in 0..amount {
        if let Err(err) = download(&client, &base_url, &dir).await {
            eprintln!("{err}");
        }
    }


    Ok(())
}

async fn download(client: &Client, base_url: &str, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let index = fastrand::u32(..MAX_INDEX);
    let url = format!("{base_url}/book/{index}");
    let resp = client
        .get(&url)
        .send()
        .await?;
    if resp.status() == 200 {
        let ext = match resp.headers().get("content-type").and_then(|v| v.to_str().ok()) {
            Some("application/zip") => "fb2.zip",
            Some("application/x-fictionbook+xml") => "fb2",
            _ => "html",
        };
        let file_name = dir.join(index.to_string()).with_extension(ext);
        let mut file = std::fs::File::create(file_name)?;
        file.write_all(&resp.bytes().await?)?;
    }


    Ok(())
}
