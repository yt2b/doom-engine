use crate::core::doom::Doom;
use crate::platform::renderer::{HEIGHT, Renderer, SCREEN_HEIGHT, SCREEN_WIDTH, WIDTH};
use anyhow::Result;
use ggez::graphics::{Image, ImageFormat};
use ggez::{
    Context, GameResult,
    conf::{WindowMode, WindowSetup},
    event::{self, EventHandler},
    graphics::{self, Mesh},
    input::keyboard::KeyCode,
};

pub struct Game {
    doom: Doom,
    renderer: Renderer,
}

impl Game {
    pub fn start(doom: Doom) -> Result<()> {
        let game = Self::new(doom)?;
        let (ctx, event_loop) = ggez::ContextBuilder::new("doom", "")
            .default_conf(ggez::conf::Conf::new())
            .window_mode(WindowMode::default().dimensions(WIDTH, HEIGHT))
            .window_setup(WindowSetup::default().title("DOOM"))
            .build()?;
        event::run(ctx, event_loop, game);
    }

    fn new(doom: Doom) -> Result<Self> {
        let graphic = doom.wad.read_graphic()?;
        Ok(Self {
            doom,
            renderer: Renderer::new(graphic),
        })
    }
}

impl EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if ctx.keyboard.is_key_pressed(KeyCode::Left) {
            self.doom.player.move_angle(2.0);
        }
        if ctx.keyboard.is_key_pressed(KeyCode::Right) {
            self.doom.player.move_angle(-2.0);
        }
        if ctx.keyboard.is_key_pressed(KeyCode::Up) {
            self.doom.player.move_pos(4.0);
        }
        if ctx.keyboard.is_key_pressed(KeyCode::Down) {
            self.doom.player.move_pos(-4.0);
        }
        self.doom.update();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut mb = graphics::MeshBuilder::new();
        self.renderer
            .draw(&mut mb, &self.doom)
            .map_err(|e| ggez::GameError::CustomError(e.to_string()))?;
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);
        let image = Image::from_pixels(
            ctx,
            &self.renderer.get_pixel_buf(),
            ImageFormat::Rgba8UnormSrgb,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
        );
        canvas.draw(
            &image,
            graphics::DrawParam::default()
                .dest([100.0, 300.0])
                .scale([1.5, 1.5]),
        );
        let mesh = Mesh::from_data(ctx, mb.build());
        canvas.draw(&mesh, graphics::DrawParam::default());
        canvas.finish(ctx)
    }
}
