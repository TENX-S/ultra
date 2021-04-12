use anyhow::{anyhow, Result};

fn main() -> Result<()> {
    if !cfg!(any(windows, unix)) {
        Err(anyhow!("Unsupported platform: {}", std::env::consts::OS))
    } else {
        Ok(())
    }
}
