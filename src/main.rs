use std::io::Write;
use std::path::{Path, PathBuf};

use reqwest::Client;


const URL_PREFFIX: &str = "https://";
const URL: &str = "searchfloor.org";

// 14.09.2026
const MAX_INDEX: u32 = 27118;


struct Args {
    amount: u32,
    directory: PathBuf,
}
impl Args {
    fn parse() -> Result<Self, lexopt::Error> {
        use lexopt::prelude::*;


        let mut a = None;
        let mut dir = None;

        let mut parser = lexopt::Parser::from_env();
        while let Some(arg) = parser.next()? {
            match arg {
                Short('a') | Long("amount") if a.is_none() => {
                    let v: u32 = parser.value()?.parse()?;
                    a = Some(v);
                },
                Short('d') | Long("directory") if dir.is_none() => {
                    let v = parser.value()?;
                    let p = PathBuf::from(v);
                    dir = Some(p);
                },

                _ => return Err(arg.unexpected())
            }
        }

        let amount = a.unwrap_or(1);
        let directory = dir.unwrap_or(PathBuf::from("."));
        Ok(Self{ amount, directory })
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = URL_PREFFIX.to_string() + URL;
    let client = reqwest::Client::new();
    let args = Args::parse()?;

    let amount = args.amount;
    let dir = args.directory;
    if !dir.is_dir() {
        use std::fs;

        if dir.is_file() {
            fs::remove_file(&dir)?;
        }
        fs::create_dir_all(&dir)?;
    }


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
