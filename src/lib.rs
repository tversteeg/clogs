use anyhow::Result;
use smart_default::SmartDefault;
use miniquad::{
    conf::{Conf, Loading},
    graphics::Context,
    EventHandler, UserData,
};

/// The main game object.
///
/// ## Example
///
/// ```rust
/// # fn main() {
/// let game = Clog::new("Title of the game")
///     .width(640)
///     .height(480);
///
/// game.start();
/// # }
/// ```
#[derive(Debug, SmartDefault)]
pub struct Clog {
    /// The window title of the game.
    title: String,

    /// The window width dimension.
    #[default = 800]
    width: i32,

    /// The window height dimension.
    #[default = 800]
    height: i32,

    /// How many MSAA samples are used for rendering the vector graphics.
    ///
    /// Defaults to 8 samples.
    #[default = 8]
    sample_count: i32,

    /// SVGs to load.
    svgs: Vec<(String, String)>,

    /// Fonts to load.
    fonts: Vec<(String, String)>,
}

impl Clog {
    /// Setup a new game.
    pub fn new<T>(title: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            title: title.into(),
            ..Default::default()
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

    /// Add a SVG that will be uploaded to the GPU during the loading phase.
    ///
    /// The `reference_name` argument can be later used in scripts to create instances of the SVG
    /// mesh.
    /// The SVG format must not violate the [usvg limitations](https://github.com/RazrFalcon/resvg/tree/master/usvg#limitations).
    pub fn load_svg<R, S>(mut self, reference_name: R, svg_source: S) -> Self
    where
        S: Into<String>,
        R: Into<String>,
    {
        self.svgs.push((reference_name.into(), svg_source.into()));

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
