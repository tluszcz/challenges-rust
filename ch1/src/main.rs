use reqwest::Error;
use std::fs::{self, File};
use std::io::{self, Write};
use futures_util::StreamExt;
use tokio::task;

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

async fn download_chunks(url: &str, f_size: u64, chunks_num: u8) -> Result<(), Error> {
    let ch_size: u64 = f_size / chunks_num as u64;
    let mut ranges: Vec<String> = vec![];

    for i in 1..=chunks_num {
        if i == 1 { 
            let r = format!("bytes=0-{}", ch_size-1);
            ranges.push(r);
            continue;
        }

        if i == chunks_num {
            let r = format!("bytes={}-", ch_size*(chunks_num-1) as u64);
            ranges.push(r);
            break;
        }

        let r = format!("bytes={}-{}",
                ch_size * (i - 1) as u64,
                (ch_size * i as u64) - 1
            );
            ranges.push(r);
    }

    let mut clients = vec![];

    for i in 0..chunks_num {
        clients.push(
            task::spawn(
                reqwest::Client::new()
                            .get(url)
                            .header("Range", ranges[i as usize].clone())
                            .send()
            )
        );
    }


    async fn stream_to_file(r: reqwest::Response, f_name: String) {
        println!("Chunk {} starting", f_name);

        let mut stream = r.bytes_stream();
        let mut data_file = File::create(f_name)
                                            .expect("Failed to create file");

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.unwrap();
            data_file.write(&*chunk).unwrap();
        }
    }

    let mut handles = vec![];

    for (i, c) in clients.into_iter().enumerate() {
        let o = c.await.unwrap().unwrap();
        handles.push(task::spawn(
            stream_to_file(o, format!("chunk{}.data", i))
        ));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let mut output = File::create("output").unwrap();
    let mut inputs = vec![];
    for n in 0..chunks_num {
        inputs.push(format!("chunk{}.data", n));
    }

    for i in inputs {
        let f_name = i.clone();
        let mut input = File::open(i).unwrap();
        io::copy(&mut input, &mut output).unwrap();
        fs::remove_file(f_name).unwrap();
    }

    println!("Download done!");

    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    let url = "https://mirror.0xem.ma/arch/iso/2025.08.01/archlinux-x86_64.iso";
    let (size, split_ok) = get_file_info(url).await?;
    println!("File: {url} : Size: {size} bytes, Accepts ranges: {split_ok}");

    let _result = download_chunks(url, size, 10).await?;

    Ok(())
}
