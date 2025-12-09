use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use futures::stream::{self, StreamExt};
use image::ImageEncoder;
use image::codecs::jpeg::JpegEncoder;
use serde::Deserialize;
use tokio::io::{self, AsyncReadExt};
use tracing::{error, info};
use url::Url;

#[derive(Debug, Parser)]
#[command(about = "Strip JPEG metadata and recompress images", version)]
struct CliArgs {
    /// Optional path to a configuration file (TOML)
    #[arg(long, env = "STEP3_CONFIG")]
    config: Option<PathBuf>,

    /// Maximum number of images processed at once
    #[arg(long, env = "STEP3_CONCURRENCY")]
    concurrency: Option<usize>,

    /// Output directory for processed images
    #[arg(long, env = "STEP3_OUTPUT_DIR")]
    output_dir: Option<PathBuf>,

    /// JPEG quality for recompressed images
    #[arg(long, env = "STEP3_QUALITY")]
    quality: Option<u8>,

    /// Direct list of inputs (files or URLs). Accepts comma-separated values from env.
    #[arg(long, short, env = "STEP3_INPUTS", value_delimiter = ',')]
    inputs: Vec<String>,

    /// Path to a file with EOL separated inputs
    #[arg(long, env = "STEP3_INPUT_FILE")]
    input_file: Option<PathBuf>,

    /// Read inputs from STDIN (EOL separated)
    #[arg(long, env = "STEP3_READ_STDIN")]
    read_stdin: bool,
}

#[derive(Debug, Deserialize, Default, Clone)]
struct FileConfig {
    concurrency: Option<usize>,
    output_dir: Option<PathBuf>,
    quality: Option<u8>,
    inputs: Option<Vec<String>>,
    input_file: Option<PathBuf>,
    read_stdin: Option<bool>,
}

#[derive(Debug, Clone)]
struct Config {
    concurrency: usize,
    output_dir: PathBuf,
    quality: u8,
    inputs: Vec<String>,
    input_file: Option<PathBuf>,
    read_stdin: bool,
}

impl Config {
    fn from_sources(cli: CliArgs) -> Result<Self> {
        let file_cfg = load_file_config(cli.config.as_deref())?;

        let output_dir = cli
            .output_dir
            .or_else(|| file_cfg.output_dir.clone())
            .unwrap_or_else(|| PathBuf::from("output"));

        let concurrency = cli
            .concurrency
            .or(file_cfg.concurrency)
            .filter(|v| *v > 0)
            .unwrap_or(4);

        let quality = cli
            .quality
            .or(file_cfg.quality)
            .map(|q| q.clamp(1, 100))
            .unwrap_or(80);

        let mut inputs: Vec<String> = Vec::new();
        inputs.extend(file_cfg.inputs.clone().unwrap_or_default());
        inputs.extend(cli.inputs.clone());

        let input_file = cli.input_file.or_else(|| file_cfg.input_file.clone());
        let read_stdin = cli.read_stdin || file_cfg.read_stdin.unwrap_or(false);

        Ok(Self {
            concurrency,
            output_dir,
            quality,
            inputs,
            input_file,
            read_stdin,
        })
    }
}

fn load_file_config(path: Option<&Path>) -> Result<FileConfig> {
    let path = match path {
        Some(path) => Some(path.to_path_buf()),
        None => {
            let default = PathBuf::from("step3.toml");
            default.exists().then_some(default)
        }
    };

    if let Some(path) = path {
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let cfg: FileConfig = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        Ok(cfg)
    } else {
        Ok(FileConfig::default())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = CliArgs::parse();
    let config = Config::from_sources(cli)?;

    tokio::fs::create_dir_all(&config.output_dir)
        .await
        .context("Failed to create output directory")?;

    let mut inputs = collect_inputs(&config).await?;
    if inputs.is_empty() {
        return Err(anyhow!("No inputs provided"));
    }

    // De-duplicate inputs to avoid repeated work
    let mut seen = HashSet::new();
    inputs.retain(|item| seen.insert(item.clone()));

    let client = reqwest::Client::new();
    let start = Instant::now();

    info!(
        "Processing {} inputs with concurrency {}",
        inputs.len(),
        config.concurrency
    );

    stream::iter(inputs.into_iter().enumerate().map(|(idx, input)| {
        let client = client.clone();
        let cfg = config.clone();
        async move {
            if let Err(err) = process_single(idx, &input, &cfg, &client).await {
                error!(target: "step3", "{}: {err:#}", input);
            }
        }
    }))
    .buffer_unordered(config.concurrency)
    .collect::<Vec<_>>()
    .await;

    info!("Completed processing in {:.2?}", start.elapsed());

    Ok(())
}

async fn collect_inputs(config: &Config) -> Result<Vec<String>> {
    let mut inputs = config.inputs.clone();

    if let Some(ref path) = config.input_file {
        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read input file: {}", path.display()))?;
        inputs.extend(
            content
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|s| !s.is_empty()),
        );
    }

    if config.read_stdin {
        let mut buffer = String::new();
        let mut stdin = io::stdin();
        stdin
            .read_to_string(&mut buffer)
            .await
            .context("Failed to read from STDIN")?;
        inputs.extend(
            buffer
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|s| !s.is_empty()),
        );
    }

    Ok(inputs)
}

async fn process_single(
    index: usize,
    input: &str,
    config: &Config,
    client: &reqwest::Client,
) -> Result<()> {
    let span_start = Instant::now();
    let data = fetch_bytes(input, client).await?;

    let format = image::guess_format(&data).context("Unable to detect image format")?;
    if format != image::ImageFormat::Jpeg {
        return Err(anyhow!("{input} is not a JPEG image"));
    }

    let image = tokio::task::spawn_blocking(move || image::load_from_memory(&data)).await??;

    let encoded = tokio::task::spawn_blocking({
        let quality = config.quality;
        move || -> Result<Vec<u8>> {
            let mut buffer = Vec::new();
            let mut encoder = JpegEncoder::new_with_quality(&mut buffer, quality);
            encoder
                .write_image(
                    image.as_bytes(),
                    image.width(),
                    image.height(),
                    image.color().into(),
                )
                .context("Failed to encode JPEG")?;
            Ok(buffer)
        }
    })
    .await??;

    let file_name = output_name(input, index);
    let destination = config.output_dir.join(file_name);
    tokio::fs::write(&destination, encoded)
        .await
        .with_context(|| format!("Failed to write image to {}", destination.display()))?;

    info!(
        target: "step3",
        "Processed {} -> {} in {:.2?}",
        input,
        destination.display(),
        span_start.elapsed()
    );

    Ok(())
}

async fn fetch_bytes(input: &str, client: &reqwest::Client) -> Result<Vec<u8>> {
    if let Ok(url) = Url::parse(input) {
        let response = client
            .get(url)
            .send()
            .await
            .context("Failed to fetch URL")?
            .error_for_status()
            .context("Non-successful status code")?;
        let bytes = response
            .bytes()
            .await
            .context("Failed to read response body")?;
        Ok(bytes.to_vec())
    } else {
        tokio::fs::read(input)
            .await
            .with_context(|| format!("Failed to read file: {input}"))
    }
}

fn output_name(input: &str, idx: usize) -> String {
    if let Ok(url) = Url::parse(input) {
        if let Some(name) = url
            .path_segments()
            .and_then(|mut segments| segments.rev().find(|s| !s.is_empty()))
        {
            return normalize_name(name);
        }
    }

    let path = Path::new(input);
    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
        return normalize_name(name);
    }

    format!("image_{idx:04}.jpg")
}

fn normalize_name(name: &str) -> String {
    if name.to_ascii_lowercase().ends_with(".jpg") || name.to_ascii_lowercase().ends_with(".jpeg") {
        name.to_string()
    } else {
        format!("{name}.jpg")
    }
}
