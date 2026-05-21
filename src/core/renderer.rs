use crate::core::doom::Doom;
use crate::core::player::PLAYER_FOV;
use crate::core::renderer::graphic::Graphic;
use crate::core::renderer::pixel_buf::PixelBuf;
use crate::core::renderer::view::ViewRenderer;

pub mod graphic;
pub mod pixel_buf;
mod solidseg;
pub mod view;

pub const SCREEN_WIDTH: usize = 320;
pub const SCREEN_HEIGHT: usize = 200;

pub struct Renderer {
    pixel_buf: PixelBuf,
    graphic: Graphic,
    view_renderer: ViewRenderer,
}

impl Renderer {
    pub fn new(graphic: Graphic) -> Self {
        Self {
            pixel_buf: PixelBuf::new(SCREEN_WIDTH, SCREEN_HEIGHT),
            graphic,
            view_renderer: ViewRenderer::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32, PLAYER_FOV),
        }
    }

    pub fn render(&mut self, doom: &Doom) {
        self.pixel_buf.clear();
        self.view_renderer
            .render(&mut self.pixel_buf, &self.graphic, &doom.map, &doom.player);
    }

    pub fn get_pixel_buf(&self) -> &PixelBuf {
        &self.pixel_buf
    }
}
