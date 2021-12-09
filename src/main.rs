use std::cmp::min;
use std::fs::File;
use std::io::Write;
// use rand;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    /// item's location
    #[structopt(short, long)]
    url: String,

    /// item's destination, if not there, derrived from input
    #[structopt(short, long)]
    output: Option<String>,
}

pub async fn prepare_progress_bar(len: u64, name: impl std::fmt::Display) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("#>-"));
    pb.set_message(format!("  {}", &name));
    pb
}

pub async fn prepare_file_name(what: &str) -> anyhow::Result<String> {
    let out = what.split_terminator('/').last();
    match out {
        Some(x) => Ok(x.to_owned()),
        None => Err(anyhow::anyhow!("can't derrive")),
    }
}

pub async fn download_file(client: &Client, url: &str, path: &str) -> anyhow::Result<()> {
    // Reqwest setup
    let res = client.get(url).send().await?;
    let total_size = res.content_length().unwrap();

    // Indicatif setup
    let pb = prepare_progress_bar(total_size, url).await;

    // download chunks
    let mut file = File::create(path)?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(format!("Downloaded {} to {}", url, path));
    Ok(())
}

// #[tokio::main]
// #[tokio::main(flavor = "current_thread")]
#[tokio::main(flavor = "multi_thread", worker_threads = 32)]
async fn main() -> anyhow::Result<()> {
    // enable automatic log gathering
    // its globally enabled
    tracing_subscriber::fmt::init();

    let opt = Opt::from_args();
    let client = reqwest::Client::new();
    let name = match opt.output {
        Some(x) => x,
        None => format!("download_{}.something", rand::random::<u64>()),
    };
    download_file(&client, opt.url.as_str(), name.as_str()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    #[tokio::test]
    async fn name_deriving_works() {
        let url = "http://something.net/harambe_meme.jpg";
        let want = "harambe_meme.jpg";
        let got = prepare_file_name(url).await.unwrap();
        assert!(want == got.as_str());
    }
}
