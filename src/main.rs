mod client;

use std::{collections::HashMap, sync::Arc};

use anyhow::{ensure, Context as _, Result};
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use const_format::formatcp;
use futures::{future::try_join_all, StreamExt as _};
use tokio::{
    fs::File,
    io::{AsyncWriteExt as _, BufWriter},
    process::Command,
    sync::Semaphore,
};

use crate::client::MyfansClient;

#[derive(Parser, Debug)]
#[clap(version = formatcp!("v{} ({})", env!("CARGO_PKG_VERSION"), env!("VERGEN_GIT_SHA")))]
pub struct Cli {
    #[clap(short, long)]
    plan_id: String,

    #[clap(short, long)]
    output: Utf8PathBuf,

    #[clap(short, long, default_value_t = 4)]
    jobs: usize,

    #[clap(short, long, env = "MYFANS_TOKEN")]
    token: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    tracing_subscriber::fmt::init();

    tokio::fs::create_dir_all(&cli.output)
        .await
        .expect("Failed to create output directory.");

    let client = MyfansClient::new(cli.token).expect("Failed to build client.");

    // すべての記事 ID を取得してくる
    let post_ids = get_all_post_ids(&client, &cli.plan_id).await.unwrap();
    tracing::info!("Fetched {} post ids.", post_ids.len());

    // 記事に含まれるすべての動画 URL を取得してくる
    let video_urls = try_join_all(post_ids.into_iter().map(|post_id| async {
        client
            .get_post_video_url(&post_id)
            .await
            .map(|url| url.map(|url| (post_id, url)))
    }))
    .await
    .expect("Failed to get video url.")
    .into_iter()
    .flatten()
    .collect::<HashMap<_, _>>();
    tracing::info!("Fetched {} video urls.", video_urls.len());

    // 動画のダウンロードを `cli.jobs` 数並行して行う
    let semaphore = Arc::new(Semaphore::new(cli.jobs));
    let output = Arc::new(cli.output);
    let _ = try_join_all(video_urls.into_iter().map(|(post_id, video_url)| {
        let semaphore = Arc::clone(&semaphore);
        let output = Arc::clone(&output);
        tokio::spawn(async move {
            let _sm = semaphore.acquire().await?;

            if video_url.ends_with(".m3u8") {
                download_m3u8_to_mp4(&post_id, &video_url, output.as_ref()).await
            } else if video_url.ends_with(".mp4") {
                download_mp4(&post_id, &video_url, output.as_ref()).await
            } else {
                tracing::info!(post_id, video_url, "Skipped.");
                Ok(())
            }
        })
    }))
    .await
    .expect("Failed to start convert.");
}

/// プランに紐付くすべての記事 ID を取得する
async fn get_all_post_ids(client: &MyfansClient, plan_id: &str) -> Result<Vec<String>> {
    let mut all_ids = vec![];
    let mut page_no = 1;
    loop {
        let (ids, next) = client
            .get_post_ids_by_plan_id(plan_id, "publish_start_at", 20, page_no)
            .await
            .with_context(|| format!("Failed to get post ids (page={page_no})."))?;
        all_ids.extend(ids);
        let Some(next) = next else {
            break;
        };
        page_no = next;
    }

    Ok(all_ids)
}

/// `ffmpeg` を呼び出して `.m3u8` を `.mp4` としてダウンロードする
async fn download_m3u8_to_mp4(
    post_id: &str,
    video_url: &str,
    output: impl AsRef<Utf8Path>,
) -> Result<()> {
    tracing::info!(post_id, video_url, "Download has started.");
    let output = output.as_ref().join(post_id).with_extension("mp4");
    if output.exists() {
        tracing::info!(post_id, video_url, "exists.");
        return Ok(());
    }

    let output = Command::new("ffmpeg")
        .args([
            "-i",
            video_url,
            "-c",
            "copy",
            "-bsf:a",
            "aac_adtstoasc",
            output.as_str(),
        ])
        .output()
        .await?;
    tracing::info!(post_id, video_url, "Download completed.");
    ensure!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

/// `.mp4` をそのままダウンロードする
async fn download_mp4(post_id: &str, video_url: &str, output: impl AsRef<Utf8Path>) -> Result<()> {
    tracing::info!(post_id, video_url, "Download has started.");

    let output = output.as_ref().join(post_id).with_extension("mp4");
    if output.exists() {
        tracing::info!(post_id, video_url, "exists.");
        return Ok(());
    }

    let mut reader = reqwest::get(video_url)
        .await
        .context("Failed to request.")?
        .error_for_status()?
        .bytes_stream();
    let file = File::create(output)
        .await
        .context("Failed to create output file.")?;
    let mut writer = BufWriter::new(file);

    while let Some(chunk) = reader.next().await {
        let chunk = chunk.context("Failed to read stream.")?;
        writer
            .write_all(&chunk)
            .await
            .context("Failed to write to stream.")?;
    }

    writer
        .flush()
        .await
        .context("Failed to flush writer stream.")?;

    tracing::info!(post_id, video_url, "Download completed.");
    Ok(())
}
