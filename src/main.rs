use anyhow::Result;

use crate::bot::Bot;

mod bot;
mod config;
mod consts;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_target(false)
        .pretty();

    let mut bot = Bot::new().await?;
    bot.run().await?;

    Ok(())
}
