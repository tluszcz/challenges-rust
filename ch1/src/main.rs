use reqwest::Error;
use std::fs::File;
use std::io::{self, Write};
use futures_util::StreamExt;
use tokio::join;

async fn get_file_info(url: &str) -> Result<(u64, bool), Error> {
    let client = reqwest::Client::new();

    let response = client
                            .head(url)
                            .send()
                            .await?;

    let result = response.headers();

    let accept_range = match result.get("accept-ranges") {
        Some(_) => true,
        None => false
    };

    let content_length = match result.get("content-length") {
        Some(v) => v.to_str()
        .unwrap_or("").parse::<u64>().unwrap(),
        None => 0,
    };

    Ok((content_length, accept_range))
}

async fn download_chunks(url: &str, f_size: u64) -> Result<(), Error> {

    let ch_size = f_size / 4;

    let range1 = format!("bytes=0-{}", ch_size-1);
    let range2 = format!("bytes={}-{}", ch_size, (ch_size*2)-1);
    let range3 = format!("bytes={}-{}", ch_size*2, (ch_size*3)-1);
    let range4 = format!("bytes={}-", ch_size*3);

    let ch_1 = reqwest::Client::new()
                        .get(url)
                        .header("Range", range1)
                        .send();
    let ch_2 = reqwest::Client::new()
                        .get(url)
                        .header("Range", range2)
                        .send();
    let ch_3 = reqwest::Client::new()
                        .get(url)
                        .header("Range", range3)
                        .send();
    let ch_4 = reqwest::Client::new()
                        .get(url)
                        .header("Range", range4)
                        .send();


    let (r1, r2, r3, r4) = join!(ch_1, ch_2, ch_3, ch_4);

    let r1 = r1?;
    let r2 = r2?;
    let r3 = r3?;
    let r4 = r4?;

    async fn stream_to_file(r: reqwest::Response, f_name: &str) {
        println!("Chunk {} starting", f_name);

        let mut stream = r.bytes_stream();
        let mut data_file = File::create(f_name)
                                            .expect("Failed to create file");

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.unwrap();
            eprintln!("{}", chunk.len());
            data_file.write(&*chunk).unwrap();
        }

        println!("Chunk {} finished downloading", f_name);

    }

    let d1 = stream_to_file(r1, "chunk1.data");
    let d2 = stream_to_file(r2, "chunk2.data");
    let d3 = stream_to_file(r3, "chunk3.data");
    let d4 = stream_to_file(r4, "chunk4.data");

    join!(d1, d2, d3, d4);

    let mut output = File::create("output").unwrap();
    let inputs = vec![
        "chunk1.data",
        "chunk2.data", 
        "chunk3.data", 
        "chunk4.data"
        ];

    for i in inputs {
        let mut input = File::open(i).unwrap();
        io::copy(&mut input, &mut output).unwrap();
    }

    println!("Download done!");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let url = "https://mirror.0xem.ma/arch/iso/2025.08.01/archlinux-x86_64.iso";
    let (size, split_ok) = get_file_info(url).await?;
    println!("File: {url} : Size: {size} bytes, Accepts ranges: {split_ok}");

    download_chunks(url, size).await?;
    Ok(())
}
