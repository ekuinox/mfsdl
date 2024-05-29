mod client;

use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Context as _, Result};
use clap::Parser;
use const_format::formatcp;
use futures::future::try_join_all;
use tokio::{process::Command, sync::Semaphore};

use crate::client::MyfansClient;

#[derive(Parser, Debug)]
#[clap(version = formatcp!("v{} ({})", env!("CARGO_PKG_VERSION"), env!("VERGEN_GIT_SHA")))]
pub struct Cli {
    #[clap(short, long)]
    plan_id: String,

    #[clap(short, long)]
    output: String,

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

    let post_ids = get_all_post_ids(&client, &cli.plan_id).await.unwrap();

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

    let semaphore = Arc::new(Semaphore::new(5));

    let output = Arc::new(cli.output);

    let _ = try_join_all(video_urls.into_iter().map(|(post_id, video_url)| {
        let semaphore = Arc::clone(&semaphore);
        let output = Arc::clone(&output);
        tokio::spawn(async move {
            let _sm = semaphore.acquire().await?;
            if output.ends_with(".m3u8") {
                download_m3u8(&post_id, &video_url, &output).await
            } else if output.ends_with(".mp4") {
                // TODO
                Ok(())
            } else {
                // Not supported.
                Ok(())
            }
        })
    }))
    .await
    .expect("Failed to start convert.");
}

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

async fn download_m3u8(post_id: &str, video_url: &str, output: &str) -> Result<()> {
    tracing::info!("starting {post_id} ({video_url}).");
    let output = Command::new("ffmpeg")
        .args([
            "-i",
            format!(r#"{video_url}"#).as_str(),
            "-c",
            "copy",
            "-bsf:a",
            "aac_adtstoasc",
            format!(r#"{output}/{post_id}.mp4"#).as_str(),
        ])
        .output()
        .await?;
    tracing::info!("finished {post_id}");
    if !output.status.success() {
        Err(anyhow!("{}", String::from_utf8_lossy(&output.stderr)))
    } else {
        Ok(()) as Result<()>
    }
}
