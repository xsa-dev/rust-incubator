use anyhow::Result;
use clap::Parser;
use futures::stream::{self, StreamExt};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::runtime::Builder;

#[derive(Debug, Parser)]
#[command(about = "Download web pages concurrently", version)]
struct Args {
    /// Maximum number of worker threads to use
    #[arg(long, default_value_t = num_cpus::get())]
    max_threads: usize,

    /// Path to a file containing newline-separated URLs
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let threads = args.max_threads.max(1);
    let runtime = Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_all()
        .build()?;

    runtime.block_on(async_main(args))?;
    Ok(())
}

async fn async_main(args: Args) -> Result<()> {
    let urls = read_urls(&args.input).await?;
    if urls.is_empty() {
        return Ok(());
    }

    let output_dir = std::env::current_dir()?;
    download_all(urls, args.max_threads.max(1), &output_dir).await?;

    Ok(())
}

async fn read_urls(path: &Path) -> Result<Vec<String>> {
    let content = tokio::fs::read_to_string(path).await?;
    Ok(content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect())
}

async fn download_all(
    urls: Vec<String>,
    max_concurrency: usize,
    output_dir: &Path,
) -> Result<Vec<PathBuf>> {
    if urls.is_empty() {
        return Ok(Vec::new());
    }

    tokio::fs::create_dir_all(output_dir).await?;
    let client = reqwest::Client::builder().no_proxy().build()?;

    let results = stream::iter(urls.into_iter().map(|url| {
        let client = client.clone();
        let dir = output_dir.to_path_buf();
        async move { download_single(&client, &url, &dir).await }
    }))
    .buffer_unordered(max_concurrency)
    .collect::<Vec<Result<PathBuf>>>()
    .await;

    results.into_iter().collect()
}

async fn download_single(client: &reqwest::Client, url: &str, dir: &Path) -> Result<PathBuf> {
    let response = client.get(url).send().await?.error_for_status()?;
    let bytes = response.bytes().await?;

    let filename = sanitize_filename(url);
    let path = dir.join(filename);
    tokio::fs::write(&path, &bytes).await?;
    Ok(path)
}

fn sanitize_filename(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let hash = hasher.finalize();
    format!("{:x}.html", hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use tokio::runtime::Runtime;

    fn create_runtime() -> Runtime {
        Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("runtime")
    }

    #[test]
    fn downloads_all_links_to_files() {
        let server = MockServer::start();
        let mock1 = server.mock(|when, then| {
            when.method(GET).path("/page1");
            then.status(200)
                .header("content-type", "text/html")
                .body("<html>one</html>");
        });
        let mock2 = server.mock(|when, then| {
            when.method(GET).path("/page2");
            then.status(200)
                .header("content-type", "text/html")
                .body("<html>two</html>");
        });

        let urls = vec![server.url("/page1"), server.url("/page2")];
        let tmp = tempfile::tempdir().expect("tempdir");
        let output_dir = tmp.path().to_path_buf();

        let rt = create_runtime();
        let paths = rt
            .block_on(download_all(urls.clone(), 2, &output_dir))
            .expect("download");

        assert_eq!(paths.len(), 2);
        mock1.assert();
        mock2.assert();

        for url in urls {
            let expected = output_dir.join(sanitize_filename(&url));
            assert!(paths.contains(&expected));
            let contents = fs::read_to_string(expected).expect("read file");
            assert!(contents.contains("<html>"));
        }
    }

    #[test]
    fn sanitize_filename_is_stable() {
        let url = "https://example.com/page";
        let first = sanitize_filename(url);
        let second = sanitize_filename(url);
        assert_eq!(first, second);
        assert!(first.ends_with(".html"));
    }
}
