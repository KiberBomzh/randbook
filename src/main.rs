#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::get("https://searchfloor.org").await?;
    dbg!(response.text().await?);


    Ok(())
}
