use anyhow::Result;
use std::io;

pub fn wait_enter() -> Result<()> {
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(())
}
