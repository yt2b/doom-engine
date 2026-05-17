use crate::core::bsp::get_subsector_indices;
use crate::core::math::{Line, Vector2};
use crate::core::player::Player;
use crate::core::renderer::graphic::{FLAT_SIZE, Graphic, SKY_ID, Texture};
use crate::core::renderer::pixel_buf::PixelBuf;
use crate::core::renderer::solidseg::SolidSeg;
use wad_reader::map::{Map, Seg, SubSector};

pub struct ViewRenderer {
    width: f32,
    half_width: f32,
    height: f32,
    half_height: f32,
    screen_dist: f32,
    solid_seg: SolidSeg,
    upper_clip: Vec<f32>,
    lower_clip: Vec<f32>,
    fov_x_to_angle: Vec<f32>,
}

impl ViewRenderer {
    pub fn new(width: f32, height: f32, fov: f32) -> Self {
        let half_width = width / 2.0;
        let screen_dist = half_width / (fov / 2.0).to_radians().tan();
        let fov_x_to_angle = (0..(width as i16))
            .map(|fov_x| convert_fov_x_to_angle(fov_x, half_width, screen_dist))
            .collect();
        Self {
            width,
            half_width,
            height,
            half_height: height / 2.0,
            screen_dist,
            solid_seg: SolidSeg::new(width as i16),
            upper_clip: vec![-1.0; width as usize],
            lower_clip: vec![height; width as usize],
            fov_x_to_angle,
        }
    }

    fn angle_to_fov_x(&self, angle: f32) -> i16 {
        (self.half_width - self.screen_dist * (angle.to_radians().tan())) as i16
    }

    pub fn render(
        &mut self,
        pixel_buf: &mut PixelBuf,
        graphic: &Graphic,
        map: &Map,
        player: &Player,
    ) {
        self.solid_seg.initialize();
        self.upper_clip.fill(-1.0);
        self.lower_clip.fill(self.height);
        // プレイヤーから見えるサブセクターを描画する
        for idx in get_subsector_indices(map, player) {
            self.render_subsector(pixel_buf, graphic, &map.subsectors[idx], map, player);
        }
    }

    pub fn render_subsector(
        &mut self,
        pixel_buf: &mut PixelBuf,
        graphic: &Graphic,
        sub_sector: &SubSector,
        map: &Map,
        player: &Player,
    ) {
        for i in 0..sub_sector.seg_count {
            let seg = &map.segs[(sub_sector.seg_idx + i) as usize];
            self.render_seg(pixel_buf, graphic, seg, map, player);
        }
    }

    pub fn render_seg(
        &mut self,
        pixel_buf: &mut PixelBuf,
        graphic: &Graphic,
        seg: &wad_reader::map::Seg,
        map: &Map,
        player: &Player,
    ) {
        let start = Vector2::new(
            map.vertexes[seg.start as usize].x as f32,
            map.vertexes[seg.start as usize].y as f32,
        );
        let end = Vector2::new(
            map.vertexes[seg.end as usize].x as f32,
            map.vertexes[seg.end as usize].y as f32,
        );
        let line = Line::new(start, end);
        if let Some((start_angle, end_angle)) = player.to_fov_line_angle(line) {
            let linedef = &map.linedefs[seg.line as usize];
            // 後ろがない場合はsolid wallとして描画する
            if linedef.back == -1 {
                let fov_x = (
                    self.angle_to_fov_x(start_angle),
                    self.angle_to_fov_x(end_angle),
                );
                let ranges = self.solid_seg.get_renderable_ranges(fov_x);
                for range in &ranges {
                    self.render_solid_wall(pixel_buf, graphic, map, seg, player, line, *range);
                }
                for range in ranges {
                    self.solid_seg.set_renderable_range(range);
                }
                return;
            }
            let front_sidedef = &map.sidedefs[seg.front_sidedef as usize];
            let back_sidedef = &map.sidedefs[seg.back_sidedef as usize];
            let front_sector = &map.sectors[front_sidedef.sector as usize];
            let back_sector = &map.sectors[back_sidedef.sector as usize];
            // 前後の部屋の天井または床の高さが違う場合は、portal wallとして描画する
            if (front_sector.ceiling_height != back_sector.ceiling_height)
                || (front_sector.floor_height != back_sector.floor_height)
            {
                let fov_x = (
                    self.angle_to_fov_x(start_angle),
                    self.angle_to_fov_x(end_angle),
                );
                for range in self.solid_seg.get_renderable_ranges(fov_x) {
                    self.render_portal_wall(pixel_buf, graphic, map, seg, player, line, range);
                }
                return;
            }
            if back_sector.ceiling_texture_name == front_sector.ceiling_texture_name
                && back_sector.floor_texture_name == front_sector.floor_texture_name
                && back_sector.light_level == front_sector.light_level
                && front_sidedef.upper_texture_name == "-"
            {
                return;
            }
            let fov_x = (
                self.angle_to_fov_x(start_angle),
                self.angle_to_fov_x(end_angle),
            );
            for range in self.solid_seg.get_renderable_ranges(fov_x) {
                self.render_portal_wall(pixel_buf, graphic, map, seg, player, line, range);
            }
        }
    }

    fn calc_scale(&self, fov_x: i16, normal_angle: f32, dist: f32, player_angle: f32) -> f32 {
        // 視界上の角度(0度なら正面)
        let angle_fov = self.fov_x_to_angle[fov_x as usize];
        // 視線と壁の法線との差
        let angle_b = (normal_angle - (player_angle + angle_fov)).abs();
        // 端点との距離(斜めの壁の補正をした後、視線の歪みを補正する)
        let edge_dist = dist / angle_b.to_radians().cos() * angle_fov.to_radians().cos();
        // 倍率
        self.screen_dist / edge_dist
    }

    fn calc_line_scale(
        &self,
        normal_angle: f32,
        normal_dist: f32,
        player_angle: f32,
        fov_x: (i16, i16),
    ) -> (f32, f32) {
        // (視界のx座標、壁の法線の角度、法線の距離、プレイヤーの角度）から倍率を求める
        let scale1 = self.calc_scale(fov_x.0, normal_angle, normal_dist, player_angle);
        let scale2 = self.calc_scale(fov_x.1, normal_angle, normal_dist, player_angle);
        let scale_step = if (fov_x.1 - fov_x.0) > 0 {
            (scale2 - scale1) / (fov_x.1 - fov_x.0) as f32
        } else {
            0.0
        };
        (scale1, scale_step)
    }

    pub fn render_solid_wall(
        &mut self,
        pixel_buf: &mut PixelBuf,
        graphic: &Graphic,
        map: &Map,
        seg: &Seg,
        player: &Player,
        line: Line,
        fov_x: (i16, i16),
    ) {
        let linedef = &map.linedefs[seg.line as usize];
        let sidedef = &map.sidedefs[seg.front_sidedef as usize];
        let sector = &map.sectors[sidedef.sector as usize];
        // テクスチャ
        let ceiling_texture = &sector.ceiling_texture_name;
        let wall_texture = &sidedef.middle_texture_name;
        let floor_texture = &sector.floor_texture_name;
        let light_level = sector.light_level;
        let texture = &graphic.textures[wall_texture];
        // プレイヤーの視点からの高さ
        let ceiling_height = sector.ceiling_height as f32 - player.view_height;
        let floor_height = sector.floor_height as f32 - player.view_height;
        // 描画判定
        let is_render_ceiling = ceiling_height > 0.0 || sector.ceiling_texture_name == SKY_ID;
        let is_render_wall = sidedef.middle_texture_name != "-";
        let is_render_floor = floor_height < 0.0;

        // 壁の法線の角度
        let normal_angle = seg.angle + 90.0;
        // 壁の法線の角度 - 壁の始点の角度
        let offset_angle = normal_angle - (line.start - player.pos).angle();
        // プレイヤーと壁の始点の距離
        let hypotenuse = line.start.dist(&player.pos);
        // 壁の法線距離 ※「cos(offset_angle) = normal_dist / hypotenuse」の変形
        let normal_dist = hypotenuse * offset_angle.to_radians().cos();
        let (scale1, scale_step) =
            self.calc_line_scale(normal_angle, normal_dist, player.angle, fov_x);
        let wall_y1 = (self.height / 2.0) - ceiling_height * scale1;
        let wall_y1_step = -scale_step * ceiling_height;
        let wall_y2 = (self.height / 2.0) - floor_height * scale1;
        let wall_y2_step = -scale_step * floor_height;
        let texture_x_offset =
            calc_texture_x_offset(hypotenuse, offset_angle, seg.offset_dist, sidedef.offset_x);
        // 壁の法線と視線の角度差
        let center_angle = normal_angle - player.angle;
        let texture_y_offset = if linedef.flags & 0x10 > 0 {
            floor_height + texture.height as f32
        } else {
            ceiling_height
        } + sidedef.offset_y as f32;

        for x in fov_x.0..(fov_x.1 + 1) {
            let idx_x = x as usize;
            let diff = (x - fov_x.0) as f32;
            let y1 = wall_y1 + wall_y1_step * diff;
            let y2 = wall_y2 + wall_y2_step * diff;
            if is_render_ceiling {
                // 天井の上端はクリップの上端、下端は壁全体の上端とクリップの下端の小さい方
                let c_y1 = self.upper_clip[idx_x] + 1.0;
                let c_y2 = (y1 - 1.0).min(self.lower_clip[idx_x] - 1.0);
                self.render_flat(
                    pixel_buf,
                    player,
                    x,
                    c_y1 as usize,
                    c_y2 as usize,
                    ceiling_texture,
                    ceiling_height,
                    light_level,
                    graphic,
                );
            }
            if is_render_wall {
                // 壁の上端は壁全体の上端をクリップ、下端は壁全体の下端をクリップして描画する
                let w_y1 = y1.max(self.upper_clip[idx_x] + 1.0);
                let w_y2 = y2.min(self.lower_clip[idx_x] - 1.0);
                // 角度差からテクスチャのどこを描画するかを決める
                let angle = center_angle - self.fov_x_to_angle[x as usize];
                let texture_column =
                    (normal_dist * angle.to_radians().tan() - texture_x_offset) as i16;
                // 倍率の逆数をかけてテクスチャの大きさを補正する
                let inverse_scale = 1.0 / (scale1 + scale_step * diff);
                self.render_texture(
                    pixel_buf,
                    x,
                    w_y1 as usize,
                    w_y2 as usize,
                    texture,
                    texture_column,
                    texture_y_offset,
                    inverse_scale,
                    light_level,
                    graphic,
                );
            }
            if is_render_floor {
                // 床の上端は壁全体の下端とクリップの上端の大きい方、下端はクリップの下端
                let f_y1 = (y2 + 1.0).max(self.upper_clip[idx_x] + 1.0);
                let f_y2 = self.lower_clip[idx_x] - 1.0;
                self.render_flat(
                    pixel_buf,
                    player,
                    x,
                    f_y1 as usize,
                    f_y2 as usize,
                    floor_texture,
                    floor_height,
                    light_level,
                    graphic,
                );
            }
        }
    }

    fn render_portal_wall(
        &mut self,
        pixel_buf: &mut PixelBuf,
        graphic: &Graphic,
        map: &Map,
        seg: &Seg,
        player: &Player,
        line: Line,
        fov_x: (i16, i16),
    ) {
        let linedef = &map.linedefs[seg.line as usize];
        let front_sector = &map.sectors[seg.front_sector as usize];
        let back_sector = &map.sectors[seg.back_sector as usize];
        let front_sidedef = &map.sidedefs[seg.front_sidedef as usize];
        let light_level = front_sector.light_level;
        // テクスチャ
        let upper_wall_texture = &front_sidedef.upper_texture_name;
        let lower_wall_texture = &front_sidedef.lower_texture_name;
        let ceiling_texture = &front_sector.ceiling_texture_name;
        let floor_texture = &front_sector.floor_texture_name;
        // 高さ
        let front_ceiling_height = front_sector.ceiling_height as f32 - player.view_height;
        let front_floor_height = front_sector.floor_height as f32 - player.view_height;
        let back_ceiling_height = back_sector.ceiling_height as f32 - player.view_height;
        let back_floor_height = back_sector.floor_height as f32 - player.view_height;
        // 空のportalなら後ろの天井の高さ
        let is_sky_portal = front_sector.ceiling_texture_name == back_sector.ceiling_texture_name
            && front_sector.ceiling_texture_name == SKY_ID;
        let front_ceiling_height = if is_sky_portal {
            back_ceiling_height
        } else {
            front_ceiling_height
        };

        let ceiling_condition = front_ceiling_height != back_ceiling_height
            || front_sector.ceiling_texture_name != back_sector.ceiling_texture_name
            || front_sector.light_level != back_sector.light_level;
        let is_render_ceiling = ceiling_condition
            && (front_ceiling_height >= 0.0 || front_sector.ceiling_texture_name == SKY_ID);
        // 手前の天井が高い場合はupper wallを描画する
        let is_render_upper_wall = ceiling_condition
            && upper_wall_texture != "-"
            && (front_ceiling_height > back_ceiling_height);
        let floor_condition = front_floor_height != back_floor_height
            || front_sector.floor_texture_name != back_sector.floor_texture_name
            || front_sector.light_level != back_sector.light_level;
        let is_render_floor = floor_condition && front_floor_height < 0.0;
        // 手前の床が低い場合はlower wallを描画する
        let is_render_lower_wall = floor_condition
            && lower_wall_texture != "-"
            && (front_floor_height < back_floor_height);
        // 描画するものがない場合は何もしない
        if !is_render_ceiling && !is_render_upper_wall && !is_render_floor && !is_render_lower_wall
        {
            return;
        }

        // 壁の法線の角度
        let normal_angle = seg.angle + 90.0;
        // 壁の法線の角度 - 壁の始点の角度
        let offset_angle = normal_angle - (line.start - player.pos).angle();
        // プレイヤーと壁の始点の距離
        let hypotenuse = line.start.dist(&player.pos);
        // 壁の法線距離 ※「cos(offset_angle) = normal_dist / hypotenuse」の変形
        let normal_dist = hypotenuse * offset_angle.to_radians().cos();
        let (scale1, scale_step) =
            self.calc_line_scale(normal_angle, normal_dist, player.angle, fov_x);
        // 壁全体の上端と下端のy座標と、y座標の変化量
        let wall_y1 = self.half_height - front_ceiling_height * scale1;
        let wall_y1_step = -scale_step * front_ceiling_height;
        let wall_y2 = self.half_height - front_floor_height * scale1;
        let wall_y2_step = -scale_step * front_floor_height;

        let upper_wall_offset = if is_render_upper_wall {
            (if linedef.flags & 0x08 > 0 {
                front_ceiling_height
            } else {
                let upper_wall_texture = &graphic.textures[upper_wall_texture];
                back_ceiling_height + upper_wall_texture.height as f32
            }) + front_sidedef.offset_y as f32
        } else {
            0.0
        };
        let lower_wall_offset = if linedef.flags & 0x10 > 0 {
            front_ceiling_height
        } else {
            back_floor_height
        } + front_sidedef.offset_y as f32;
        let texture_x_offset = calc_texture_x_offset(
            hypotenuse,
            offset_angle,
            seg.offset_dist,
            front_sidedef.offset_x,
        );
        // 壁の法線と視線の角度差
        let center_angle = normal_angle - player.angle;

        // portalの上端のy座標と、y座標の変化量
        let (portal_y1, portal_y1_step) =
            if is_render_upper_wall && back_ceiling_height > front_floor_height {
                // portalの上端は後ろの天井の高さ
                (
                    self.half_height - back_ceiling_height * scale1,
                    -scale_step * back_ceiling_height,
                )
            } else {
                // portalはないので上端は壁全体の下端
                (wall_y2, wall_y2_step)
            };
        // portalの下端のy座標と、y座標の変化量
        let (portal_y2, portal_y2_step) =
            if is_render_lower_wall && back_floor_height < front_ceiling_height {
                // portalの下端は後ろの床の高さ
                (
                    self.half_height - back_floor_height * scale1,
                    -scale_step * back_floor_height,
                )
            } else {
                // portalはないので下端は壁全体の上端
                (wall_y1, wall_y1_step)
            };

        for x in fov_x.0..(fov_x.1 + 1) {
            let idx_x = x as usize;
            let diff = (x - fov_x.0) as f32;
            let wall_y1 = wall_y1 + wall_y1_step * diff;
            let wall_y2 = wall_y2 + wall_y2_step * diff;
            // 角度差からテクスチャのどこを描画するかを決める
            let angle = center_angle - self.fov_x_to_angle[x as usize];
            let texture_column = (normal_dist * angle.to_radians().tan() - texture_x_offset) as i16;
            // 倍率の逆数をかけてテクスチャの大きさを補正する
            let inverse_scale = 1.0 / (scale1 + scale_step * diff);

            if is_render_upper_wall {
                let portal_y1 = portal_y1 + portal_y1_step * diff;
                // upper_wallの上端は壁全体の上端
                let upper_wall_y1 = wall_y1;
                // upper_wallの下端はportalの上端
                let upper_wall_y2 = portal_y1;
                if is_render_ceiling {
                    // 天井の上端はクリップの上端、下端は壁全体の上端とクリップの下端の小さい方
                    let c_y1 = self.upper_clip[idx_x] + 1.0;
                    let c_y2 = (wall_y1 - 1.0).min(self.lower_clip[idx_x] - 1.0);
                    self.render_flat(
                        pixel_buf,
                        player,
                        x,
                        c_y1 as usize,
                        c_y2 as usize,
                        ceiling_texture,
                        front_ceiling_height,
                        light_level,
                        graphic,
                    );
                }
                // upper_wallの上端と下端をクリップして描画する
                let w_y1 = upper_wall_y1.max(self.upper_clip[idx_x] + 1.0);
                let w_y2 = upper_wall_y2.min(self.lower_clip[idx_x] - 1.0);
                self.render_texture(
                    pixel_buf,
                    x,
                    w_y1 as usize,
                    w_y2 as usize,
                    &graphic.textures[upper_wall_texture],
                    texture_column,
                    upper_wall_offset,
                    inverse_scale,
                    light_level,
                    graphic,
                );
                if self.upper_clip[idx_x] < w_y2 {
                    self.upper_clip[idx_x] = w_y2
                }
            }
            if is_render_ceiling {
                // 天井の上端はクリップの上端
                let c_y1 = self.upper_clip[idx_x] + 1.0;
                // 天井の下端は壁全体の上端とクリップの下端の小さい方
                let c_y2 = (wall_y1 - 1.0).min(self.lower_clip[idx_x] - 1.0);
                self.render_flat(
                    pixel_buf,
                    player,
                    x,
                    c_y1 as usize,
                    c_y2 as usize,
                    ceiling_texture,
                    front_ceiling_height,
                    light_level,
                    graphic,
                );
                if self.upper_clip[idx_x] < c_y2 {
                    self.upper_clip[idx_x] = c_y2
                }
            }
            if is_render_lower_wall {
                if is_render_floor {
                    // 床の上端は壁全体の下端とクリップの上端の大きい方、下端はクリップの下端
                    let f_y1 = (wall_y2 + 1.0).max(self.upper_clip[idx_x] + 1.0);
                    let f_y2 = self.lower_clip[idx_x] - 1.0;
                    self.render_flat(
                        pixel_buf,
                        player,
                        x,
                        f_y1 as usize,
                        f_y2 as usize,
                        floor_texture,
                        front_floor_height,
                        light_level,
                        graphic,
                    );
                }
                let portal_y2 = portal_y2 + portal_y2_step * diff;
                // lower_wallの上端はportalの下端とクリップの上端の大きい方
                let w_y1 = (portal_y2 - 1.0).max(self.upper_clip[idx_x] + 1.0);
                // lower_wallの下端は壁全体の下端とクリップの下端の小さい方
                let w_y2 = wall_y2.min(self.lower_clip[idx_x] - 1.0);
                self.render_texture(
                    pixel_buf,
                    x,
                    w_y1 as usize,
                    w_y2 as usize,
                    &graphic.textures[lower_wall_texture],
                    texture_column,
                    lower_wall_offset,
                    inverse_scale,
                    light_level,
                    graphic,
                );
                if self.lower_clip[idx_x] > w_y1 {
                    self.lower_clip[idx_x] = w_y1
                }
            }
            if is_render_floor {
                // 床の上端は壁全体の下端とクリップの上端の大きい方
                let f_y1 = (wall_y2 + 1.0).max(self.upper_clip[idx_x] + 1.0);
                // 床の下端はクリップの下端
                let f_y2 = self.lower_clip[idx_x] - 1.0;
                self.render_flat(
                    pixel_buf,
                    player,
                    x,
                    f_y1 as usize,
                    f_y2 as usize,
                    floor_texture,
                    front_floor_height,
                    light_level,
                    graphic,
                );
                if self.lower_clip[idx_x] > wall_y2 + 1.0 {
                    self.lower_clip[idx_x] = f_y1
                }
            }
        }
    }

    fn render_texture(
        &mut self,
        pixel_buf: &mut PixelBuf,
        x: i16,
        y1: usize,
        y2: usize,
        texture: &Texture,
        texture_column: i16,
        texture_y_offset: f32,
        inverse_scale: f32,
        light_level: i16,
        graphic: &Graphic,
    ) {
        if y1 > y2 {
            return;
        }
        let color_idx = ((255 - light_level) / 16) as usize;
        let texture_x = texture_column.rem_euclid(texture.width as i16) as usize;
        let mut texture_y = texture_y_offset + (y1 as f32 - self.half_height) * inverse_scale;
        for y in y1..y2 + 1 {
            let idx = (texture_y as usize % texture.height) * texture.width + texture_x;
            if let Some(palette_idx) = texture.palettes[idx] {
                let mapped_idx = graphic.colormaps[color_idx][palette_idx];
                let rgb = graphic.palettes[0][mapped_idx];
                pixel_buf.set_pixel(x as usize, y, rgb);
            }
            texture_y += inverse_scale
        }
    }

    fn render_flat(
        &mut self,
        pixel_buf: &mut PixelBuf,
        player: &Player,
        x: i16,
        y1: usize,
        y2: usize,
        texture_name: &str,
        world_height: f32,
        light_level: i16,
        graphic: &Graphic,
    ) {
        if y1 > y2 {
            return;
        }
        if texture_name == SKY_ID {
            self.render_sky_texture(
                pixel_buf,
                player.angle,
                x,
                y1,
                y2,
                &graphic.textures["SKY1"],
                graphic,
            );
        } else {
            self.render_flat_texture(
                pixel_buf,
                player,
                x,
                y1,
                y2,
                &graphic.flats[texture_name],
                world_height,
                light_level,
                graphic,
            );
        }
    }

    fn render_sky_texture(
        &mut self,
        pixel_buf: &mut PixelBuf,
        player_angle: f32,
        x: i16,
        y1: usize,
        y2: usize,
        texture: &Texture,
        graphic: &Graphic,
    ) {
        // 4つの空の画像を360度に対応させる
        let normal =
            (4.0 * (player_angle + self.fov_x_to_angle[x as usize])).rem_euclid(360.0) / 360.0;
        let texture_x = (normal * texture.width as f32) as usize;
        let inverse_scale = 160.0 / self.height;
        let mut texture_y = 100.0 + (y1 as f32 - self.half_height) * inverse_scale;
        for y in y1..y2 + 1 {
            let tex_y = texture_y.rem_euclid(texture.height as f32) as usize;
            let idx = tex_y * texture.width + texture_x;
            if let Some(pallete_idx) = texture.palettes[idx] {
                let rgb = graphic.palettes[0][pallete_idx];
                pixel_buf.set_pixel(x as usize, y, rgb);
            }
            texture_y += inverse_scale
        }
    }

    fn render_flat_texture(
        &mut self,
        pixel_buf: &mut PixelBuf,
        player: &Player,
        x: i16,
        y1: usize,
        y2: usize,
        palettes: &[usize],
        view_height: f32,
        light_level: i16,
        graphic: &Graphic,
    ) {
        let color_idx = ((255 - light_level) / 16) as usize;
        let dir_x = player.angle.to_radians().cos();
        let dir_y = player.angle.to_radians().sin();
        for y in y1..y2 + 1 {
            // プレイヤーが見ている距離
            let dist = self.half_width * view_height / (self.half_height - y as f32);
            // プレイヤーから見たワールドの点の座標
            let world_x = dir_x * dist + player.pos.x;
            let world_y = dir_y * dist + player.pos.y;
            // 視界の左端と右端のワールドの点の座標
            let left_x = -dir_y * dist + world_x;
            let left_y = dir_x * dist + world_y;
            let right_x = dir_y * dist + world_x;
            let right_y = -dir_x * dist + world_y;
            // 現在の点からテクスチャのどこを描画するかを決める
            let dx = (right_x - left_x) / self.width;
            let dy = (right_y - left_y) / self.width;
            let texture_x = (left_x + dx * x as f32).rem_euclid(FLAT_SIZE as f32) as usize;
            let texture_y = (left_y + dy * x as f32).rem_euclid(FLAT_SIZE as f32) as usize;
            let palette_idx = palettes[texture_y * FLAT_SIZE + texture_x];
            let mapped_idx = graphic.colormaps[color_idx][palette_idx];
            let rgb = graphic.palettes[0][mapped_idx];
            pixel_buf.set_pixel(x as usize, y, rgb);
        }
    }
}

fn convert_fov_x_to_angle(fov_x: i16, half_width: f32, screen_dist: f32) -> f32 {
    let x = half_width - fov_x as f32;
    (x / screen_dist).atan().to_degrees()
}

fn calc_texture_x_offset(
    hypotenuse: f32,
    offset_angle: f32,
    seg_offset_dist: i16,
    sidedef_offset_x: i16,
) -> f32 {
    // 法線との交点から端点までの距離 ※「sin(offset_angle) = opposite / hypotenuse」の変形
    let opposite = hypotenuse * offset_angle.to_radians().sin();
    // oppositeにテクスチャのオフセットを引いたものが、テクスチャのどこを描画するかの基準になる
    opposite - (seg_offset_dist + sidedef_offset_x) as f32
}

#[cfg(test)]
mod tests {
    use super::{calc_texture_x_offset, convert_fov_x_to_angle};

    #[test]
    fn test_convert_fov_x_to_angle() {
        assert_eq!(convert_fov_x_to_angle(0, 160.0, 160.0), 45.0);
        assert_eq!(convert_fov_x_to_angle(160, 160.0, 160.0), 0.0);
        assert_eq!(convert_fov_x_to_angle(320, 160.0, 160.0), -45.0);
    }

    #[test]
    fn test_get_texture_x_offset() {
        assert_eq!(calc_texture_x_offset(5.0, 0.0, 0, 0), 0.0);
        assert_eq!(calc_texture_x_offset(5.0, 0.0, 10, 0), -10.0);
        assert_eq!(calc_texture_x_offset(5.0, 0.0, 10, 10), -20.0);
        assert_eq!(calc_texture_x_offset(10.0, 30.0, 0, 0), 5.0);
        assert_eq!(calc_texture_x_offset(10.0, 30.0, 10, 10), -15.0);
        assert_eq!(calc_texture_x_offset(10.0, -30.0, 0, 0), -5.0);
        assert_eq!(calc_texture_x_offset(10.0, -30.0, 10, 10), -25.0);
    }
}
