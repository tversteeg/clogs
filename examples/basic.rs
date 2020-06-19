use anyhow::Result;
use clogs::Clog;

fn main() -> Result<()> {
    let clog = Clog::new(800, 600)?;

    clog.start();

    Ok(())
}
