use anyhow::Result;
use miniquad::{
    conf::{Conf, Loading},
    graphics::Context,
    EventHandler, UserData,
};

/// The main game object.
pub struct Clog {
    /// The window width & height.
    window_dimensions: (i32, i32),
}

impl Clog {
    /// Setup a new game.
    pub fn new(width: i32, height: i32) -> Result<Self> {
        Ok(Self {
            window_dimensions: (width, height),
        })
    }

    /// Start the game.
    pub fn start(self) {
        miniquad::start(
            Conf {
                window_title: concat!("clog - ", env!("CARGO_PKG_VERSION")).to_string(),
                window_width: self.window_dimensions.0,
                window_height: self.window_dimensions.1,
                loading: Loading::Embedded,
                sample_count: 8,
                ..Default::default()
            },
            |ctx| UserData::owning(self, ctx),
        );
    }
}

impl EventHandler for Clog {
    fn update(&mut self, _: &mut Context) {
        todo!()
    }

    fn draw(&mut self, _: &mut Context) {
        todo!()
    }
}
