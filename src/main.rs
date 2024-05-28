mod client;

use clap::Parser;

use crate::client::MyfansClient;

#[derive(Parser, Debug)]
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

    let mut all_ids = vec![];
    let mut page_no = 1;
    loop {
        let (ids, next) = client
            .post_ids_by_plan_id(&cli.plan_id, "publish_start_at", 20, page_no)
            .await
            .expect("Failed to get post ids.");
        all_ids.extend(ids);
        let Some(next) = next else {
            break;
        };
        page_no = next;
    }

    for id in &all_ids {
        println!("{id}");
    }

    println!("ids = {}", all_ids.len());
}
