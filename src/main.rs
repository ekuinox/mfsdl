mod client;

use anyhow::{Context as _, Result};
use clap::Parser;
use const_format::formatcp;

use crate::client::MyfansClient;

#[derive(Parser, Debug)]
#[clap(version = formatcp!("v{} ({})", env!("CARGO_PKG_VERSION"), env!("VERGEN_GIT_SHA")))]
pub struct Cli {
    #[clap(short, long)]
    plan_id: String,

    #[clap(short, long, env = "MYFANS_TOKEN")]
    token: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let client = MyfansClient::new(cli.token).expect("Failed to build client.");

    let post_ids = get_all_post_ids(&client, &cli.plan_id).await.unwrap();

    println!("ids = {}", post_ids.len());

    for post_id in &post_ids {
        let url = client
            .get_post_video_url(post_id)
            .await
            .expect("Failed to get post video url");
        println!("post_id({post_id}) - {}", url.unwrap_or_default());
    }
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
