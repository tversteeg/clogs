use anyhow::Result;
use miniquad::{
    conf::{Conf, Loading},
    graphics::Context,
    EventHandler, UserData,
};

/// The main game object.
///
/// ## Example
///
/// #fn main() {
/// let game = Clog::default()
///     .width(800)
///     .height(600);
///
/// game.start();
/// #}
pub struct Clog {
    /// The window title of the game.
    title: String,

    /// The window dimensions.
    ///
    /// Defaults to 800x600.
    width: i32,
    height: i32,

    /// How many MSAA samples are used for rendering the vector graphics.
    ///
    /// Defaults to 8 samples.
    sample_count: i32,
}

impl Clog {
    /// Setup a new game.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            width: 800,
            height: 600,
            sample_count: 8,
        }
    }

    /// Set the initial window width.
    pub fn width(mut self, width: i32) -> Self {
        self.width = width;

        self
    }

    /// Set the initial window height.
    pub fn height(mut self, height: i32) -> Self {
        self.height = height;

        self
    }

    /// Set how many MSAA samples are used for rendering the vector graphics.
    pub fn sample_count(mut self, sample_count: i32) -> Self {
        self.sample_count = sample_count;

        self
    }

    /// Start the game.
    pub fn start(self) {
        miniquad::start(
            Conf {
                window_title: self.title.clone(),
                window_width: self.width,
                window_height: self.height,
                loading: Loading::Embedded,
                sample_count: self.sample_count,
                ..Default::default()
            },
            |ctx| UserData::owning(self, ctx),
        );
    }
}

impl EventHandler for Clog {
    fn update(&mut self, _: &mut Context) {}

    fn draw(&mut self, _: &mut Context) {}
}
