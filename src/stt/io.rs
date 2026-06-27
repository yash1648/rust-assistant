use anyhow::Result;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

/// Block until the user presses Enter.
pub fn wait_enter() -> Result<()> {
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(())
}

/// Spawn a background thread that waits for Enter and sets a flag.
/// Returns the flag handle and the join handle (flag set to true when Enter pressed).
pub fn spawn_enter_listener() -> (Arc<AtomicBool>, thread::JoinHandle<()>) {
    let flag = Arc::new(AtomicBool::new(false));
    let flag_clone = Arc::clone(&flag);
    let handle = thread::spawn(move || {
        let mut s = String::new();
        let _ = io::stdin().read_line(&mut s);
        flag_clone.store(true, Ordering::SeqCst);
    });
    (flag, handle)
}
