mod stt;
mod tts;
mod assistant;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut assistant = assistant::Assistant::new()?;
    assistant.run().await?;
    Ok(())
}