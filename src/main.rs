use std::path::PathBuf;

use reqwest::Client;
use tokio::io::AsyncWriteExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};


const URL_PREFFIX: &str = "https://";
const URL: &str = "searchfloor.org";

// 14.09.2026
const MAX_INDEX: u32 = 27118;


const PROGRESS_BAR_TEMPLATE: &str =
    " [{bar:20}] {bytes}/{total_bytes} ({bytes_per_sec}) ";

const MAIN_PROGRESS_BAR_TEMPLATE: &str =
    " {elapsed_precise} {bar:40} {human_pos}/{human_len} ";



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
async fn main() {
    let app = match App::new() {
        Ok(a) => a,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        },
    };

    if let Err(err) = app.run().await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

struct App {
    args: Args,
    client: Client,
    base_url: String,
    progress: MultiProgress,
}
impl App {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            args: Args::parse()?,
            client: reqwest::Client::new(),
            base_url: URL_PREFFIX.to_string() + URL,
            progress: MultiProgress::new(),
        })
    }
    async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = &self.args.directory;
        if !dir.is_dir() {
            use std::fs;

            if dir.is_file() {
                fs::remove_file(dir)?;
            }
            fs::create_dir_all(dir)?;
        }


        let main_pb = self.progress.add(ProgressBar::new(self.args.amount.into()));
        if self.args.amount > 1 {
            main_pb.set_style(ProgressStyle::default_bar()
                .template(MAIN_PROGRESS_BAR_TEMPLATE).unwrap()
            );
            main_pb.enable_steady_tick(std::time::Duration::from_millis(200));
        }

        for _ in 0..self.args.amount {
            let _ = self.download().await;
            main_pb.inc(1);
        }
        main_pb.finish_and_clear();


        Ok(())
    }

    async fn download(&self) -> Result<(), Box<dyn std::error::Error>> {
        let index = fastrand::u32(..MAX_INDEX);
        let url = format!("{}/book/{index}", self.base_url);
        let dir = &self.args.directory;

        let mut resp = self.client
            .get(&url)
            .send()
            .await?;
        if resp.status() != 200 {
            self.progress.println(format!("{index}: response status isn't 200!"))?;
            return Err("".into());
        }
        let total_size = resp.content_length().unwrap_or(0);
        let pb = self.progress.add(ProgressBar::new(total_size));
        pb.set_style(ProgressStyle::default_bar()
            .template(PROGRESS_BAR_TEMPLATE).unwrap()
            .progress_chars("=> ")
        );

        let ext = match resp.headers().get("content-type").and_then(|v| v.to_str().ok()) {
            Some("application/zip") => "fb2.zip",
            Some("application/x-fictionbook+xml") => "fb2",
            _ => "html",
        };
        let file_name = dir.join(index.to_string()).with_extension(ext);
        let mut file = tokio::fs::File::create(file_name).await?;
        while let Some(chunk) = resp.chunk().await? {
            file.write_all(&chunk).await?;
            file.flush().await?;

            pb.inc(chunk.len() as u64);
        }
        pb.finish_and_clear();


        Ok(())
    }
}

