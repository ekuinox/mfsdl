mod client;
mod downloader;

use std::{collections::HashMap, sync::Arc};

use anyhow::{Context as _, Result};
use camino::Utf8PathBuf;
use clap::Parser;
use const_format::formatcp;
use futures::future::try_join_all;
use tokio::sync::Semaphore;

use crate::{client::MyfansClient, downloader::download};

#[derive(Parser, Debug)]
#[clap(version = formatcp!("v{} ({})", env!("CARGO_PKG_VERSION"), env!("VERGEN_GIT_SHA")))]
pub struct Cli {
    #[clap(short, long)]
    plan_id: String,

    #[clap(short, long)]
    output: Utf8PathBuf,

    #[clap(short, long, default_value_t = 4)]
    jobs: usize,

    /// from cookie `_mfans_token`
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

    let futures = video_urls.into_iter().map(|(post_id, video_url)| {
        let semaphore = Arc::clone(&semaphore);
        let output = Arc::clone(&output);
        async move {
            let _sm = semaphore.acquire().await?;
            download(&post_id, &video_url, output.as_ref()).await
        }
    });

    try_join_all(futures).await.expect("Failed to convert");
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
