use reqwest::Error;

async fn get_range_request(url: &str) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let response = client
                            .get(url)
                            .header("Range", "bytes=-1024")
                            .send()
                            .await?;

    println!("Status: {}", response.status());

    let bytes = response.bytes().await?;
    println!("{}", bytes.len());

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let url = todo!();
    get_range_request(url).await?;
    Ok(())
}
