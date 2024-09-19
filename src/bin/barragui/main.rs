use ggez::{event, graphics, Context, GameResult};
use std::fs;
use usvg::{Options, Tree};
use resvg::tiny_skia::Pixmap;

struct GameState {
    svg_image: Option<graphics::Image>,
}

impl GameState {
    fn new(ctx: &mut Context) -> GameResult<GameState> {
        // Load and rasterize the SVG
        let svg_path = "assets/Zwei.svg";
        let svg_image = load_svg(ctx, svg_path)?;

        Ok(GameState {
            svg_image: Some(svg_image),
        })
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::WHITE);

        // Draw the SVG image if it exists
        if let Some(ref svg_image) = self.svg_image {
            let draw_params = graphics::DrawParam::default()
                .dest([50.0, 50.0]); // Set the position to draw the image
            graphics::draw(ctx, svg_image, draw_params);
        }

        graphics::present(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    let (ctx, event_loop) = &mut ggez::ContextBuilder::new("SVG Example", "Author")
        .build()?;

    let state = &mut GameState::new(ctx)?;
    event::run(ctx, event_loop, state)
}

/// Loads an SVG file and rasterizes it into a ggez Image
fn load_svg(ctx: &mut Context, svg_path: &str) -> GameResult<graphics::Image> {
    // Load the SVG file into a string
    let svg_data = fs::read_to_string(svg_path).expect("Failed to read SVG file");

    // Parse the SVG
    let opt = Options::default();
    let rtree = Tree::from_str(&svg_data, &opt).expect("Failed to parse SVG");

    // Set the target width and height
    let width = 100;
    let height = 1000;

    // Rasterize the SVG into a Pixmap (bitmap image)
    let mut pixmap = Pixmap::new(width, height).expect("Failed to create pixmap");
    resvg::render(&rtree, usvg::FitTo::Original, &mut pixmap.as_mut());

    // Convert the pixmap into a ggez Image
    let image = graphics::Image::from_pixels(
        ctx,
        &pixmap.data(),
        graphics::ImageFormat::Rgba8UnormSrgb,
        width,
        height,
    );

    Ok(image)
}
