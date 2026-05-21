use anyhow::Result;

use crate::core::bsp::get_subsector_indices;
use crate::core::map::{Map, Position};
use crate::core::math::{Line, Vector2};
use crate::core::{doom::Doom, player::Player};
use ggez::{
    glam,
    graphics::{Color, DrawMode, FillOptions, MeshBuilder},
};

pub const WIDTH: f32 = 1280.0;
pub const HEIGHT: f32 = 800.0;

pub struct MapRenderer {
    offset_x: f32,
    offset_y: f32,
    scale: f32,
}

impl MapRenderer {
    pub fn new(offset_x: f32, offset_y: f32, scale: f32) -> Self {
        Self {
            offset_x,
            offset_y,
            scale,
        }
    }

    pub fn render(&self, mb: &mut MeshBuilder, doom: &Doom) -> Result<()> {
        self.render_map(mb, &doom.map, Color::from_rgb(64, 64, 64))?;
        self.render_player(mb, &doom.player, Color::GREEN)?;
        self.render_insight_subsector(mb, &doom.player, &doom.map, Color::YELLOW)?;
        Ok(())
    }

    fn to_screen_vertex(&self, x: f32, y: f32) -> (f32, f32) {
        let x = x * self.scale + self.offset_x;
        let y = -y * self.scale + self.offset_y;
        (x, y)
    }

    fn render_line_pos(
        &self,
        mb: &mut MeshBuilder,
        line: (Position, Position),
        color: Color,
    ) -> Result<()> {
        let (s, e) = line;
        let (x1, y1) = self.to_screen_vertex(s.x as f32, s.y as f32);
        let (x2, y2) = self.to_screen_vertex(e.x as f32, e.y as f32);
        render_line(mb, x1, y1, x2, y2, color)?;
        Ok(())
    }

    fn render_line_vec2(
        &self,
        mb: &mut MeshBuilder,
        line: (Vector2, Vector2),
        color: Color,
    ) -> Result<()> {
        let (s, e) = line;
        let (x1, y1) = self.to_screen_vertex(s.x, s.y);
        let (x2, y2) = self.to_screen_vertex(e.x, e.y);
        render_line(mb, x1, y1, x2, y2, color)?;
        Ok(())
    }

    pub fn render_map(&self, mb: &mut MeshBuilder, map: &Map, color: Color) -> Result<()> {
        for l in &map.linedefs {
            let s = map.vertexes[l.start as usize];
            let e = map.vertexes[l.end as usize];
            self.render_line_pos(mb, (s, e), color)?;
        }
        for t in &map.things {
            let (x, y) = self.to_screen_vertex(t.x as f32, t.y as f32);
            render_circle(mb, x, y, 1.0, color)?;
        }
        Ok(())
    }

    pub fn render_player(&self, mb: &mut MeshBuilder, player: &Player, color: Color) -> Result<()> {
        let (px, py) = self.to_screen_vertex(player.pos.x, player.pos.y);
        render_circle(mb, px, py, 2.0, color)?;
        let half_fov = player.fov / 2.0;
        for fov in [half_fov, -half_fov] {
            let rotated = Vector2::new(800.0, 0.0).rotate(player.angle + fov);
            let dest = Vector2::new(player.pos.x + rotated.x, player.pos.y + rotated.y);
            self.render_line_vec2(mb, (player.pos, dest), color)?;
        }
        Ok(())
    }

    fn render_sub_sector(
        &self,
        mb: &mut MeshBuilder,
        idx: usize,
        map: &Map,
        player: &Player,
        color: Color,
    ) -> Result<()> {
        let sub_sector = &map.subsectors[idx];
        for i in 0..sub_sector.seg_count {
            let seg = &map.segs[(sub_sector.seg_idx + i) as usize];
            let start = Vector2::new(
                map.vertexes[seg.start as usize].x as f32,
                map.vertexes[seg.start as usize].y as f32,
            );
            let end = Vector2::new(
                map.vertexes[seg.end as usize].x as f32,
                map.vertexes[seg.end as usize].y as f32,
            );
            let line = Line::new(start, end);
            if player.is_insight_line(line) {
                self.render_line_vec2(mb, (start, end), color)?;
            }
        }
        Ok(())
    }

    pub fn render_insight_subsector(
        &self,
        mb: &mut MeshBuilder,
        player: &Player,
        map: &Map,
        color: Color,
    ) -> Result<()> {
        let indices = get_subsector_indices(map, player);
        for idx in indices {
            self.render_sub_sector(mb, idx, map, player, color)?;
        }
        Ok(())
    }
}

fn render_line(
    mb: &mut MeshBuilder,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: Color,
) -> Result<()> {
    mb.line(&[glam::vec2(x1, y1), glam::vec2(x2, y2)], 1.0, color)?;
    Ok(())
}

fn render_circle(mb: &mut MeshBuilder, x: f32, y: f32, radius: f32, color: Color) -> Result<()> {
    mb.circle(
        DrawMode::Fill(FillOptions::default()),
        glam::vec2(x, y),
        radius,
        0.1,
        color,
    )?;
    Ok(())
}
