use anyhow::Result;
use tracing_subscriber::EnvFilter;

use crate::bot::Bot;

mod bot;
mod config;
mod consts;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_target(false)
        .pretty()
        .init();

    let mut bot = Bot::new().await?;
    bot.run().await?;

    Ok(())
}
