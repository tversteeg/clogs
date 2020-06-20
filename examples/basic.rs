use anyhow::Result;
use clogs::Clog;

fn main() -> Result<()> {
    let game = Clog::new(concat!("clog - ", env!("CARGO_PKG_VERSION")))
        .width(800)
        .height(600);

    game.start();

    Ok(())
}
