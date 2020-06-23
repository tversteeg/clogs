use anyhow::Result;
use clogs::Clog;

fn main() -> Result<()> {
    // Create a new game
    let game = Clog::new(concat!("clog - ", env!("CARGO_PKG_VERSION")))
        // Load 'basic.glsp' as the main script
        .main_script(include_str!("basic.glsp"))?
        // Set the initial window dimensions
        .width(800)
        .height(600);

    game.start();

    Ok(())
}
