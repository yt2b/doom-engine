use crate::core::{
    math::Vector2,
    renderer::{
        SCREEN_HEIGHT, SCREEN_WIDTH,
        graphic::{FLAT_SIZE, Graphic},
        pixel_buf::PixelBuf,
        view::wrap_texture_coord,
    },
};

const MAX_VISPLANES: usize = 128;
const FLAT_MASK: i16 = (FLAT_SIZE - 1) as i16;

pub struct Plane {
    pub visplanes: Vec<Visplane>,
    active_indices: Vec<usize>,
    free_indices: Vec<usize>,
    span_start: [usize; SCREEN_HEIGHT],
    fov_x_to_angle: [f32; SCREEN_WIDTH],
    dist_scales: [f32; SCREEN_WIDTH],
    caches: Vec<Cache>,
    player_pos: Vector2,
    player_angle: f32,
    half_width: f32,
    half_height: f32,
    sky_inverse_scale: f32,
}

impl Plane {
    pub fn new(fov_x_to_angle: &[f32]) -> Self {
        let visplanes = (0..MAX_VISPLANES).map(|_| Visplane::default()).collect();
        let mut new_fov_x_to_angle = [0.0; SCREEN_WIDTH];
        let mut dist_scales = [0.0; SCREEN_WIDTH];
        for x in 0..SCREEN_WIDTH {
            let angle = fov_x_to_angle[x];
            new_fov_x_to_angle[x] = angle;
            dist_scales[x] = 1.0 / angle.to_radians().cos();
        }
        let (half_width, half_height) = (SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 / 2.0);
        let mut caches = Vec::new();
        for y in 0..SCREEN_HEIGHT {
            caches.push(Cache::new(half_width / (half_height - y as f32)));
        }
        Self {
            visplanes,
            active_indices: Vec::new(),
            free_indices: (0..MAX_VISPLANES).collect(),
            span_start: [0; SCREEN_HEIGHT],
            fov_x_to_angle: new_fov_x_to_angle,
            dist_scales,
            caches,
            player_pos: Vector2::new(0.0, 0.0),
            player_angle: 0.0,
            half_width: SCREEN_WIDTH as f32 / 2.0,
            half_height: SCREEN_HEIGHT as f32 / 2.0,
            sky_inverse_scale: 160.0 / (SCREEN_HEIGHT as f32),
        }
    }

    pub fn initialize(&mut self) {
        for visplane in self.visplanes.iter_mut() {
            visplane.initialize();
        }
        while let Some(idx) = self.active_indices.pop() {
            self.free_indices.push(idx);
        }
        for cache in self.caches.iter_mut() {
            cache.initialize();
        }
    }

    pub fn set_player_state(&mut self, player_pos: Vector2, player_angle: f32) {
        self.player_pos = player_pos;
        self.player_angle = player_angle;
    }

    pub fn get_visplane_idx(
        &mut self,
        height: f32,
        flat_id: usize,
        lightlevel: i16,
        fov_x: (i16, i16),
    ) -> usize {
        // 既存のvisplaneを検索
        for idx in self.active_indices.iter() {
            let visplane = &self.visplanes[*idx];
            if visplane.height == height
                && visplane.flat_id == flat_id
                && visplane.light_level == lightlevel
                && visplane.validte(fov_x)
            {
                let visplane = &mut self.visplanes[*idx];
                visplane.fov_x = (fov_x.0.min(visplane.fov_x.0), fov_x.1.max(visplane.fov_x.1));
                return *idx;
            }
        }
        // 新しいvisplaneを割り当てる
        if let Some(idx) = self.free_indices.pop() {
            let visplane = &mut self.visplanes[idx];
            visplane.height = height;
            visplane.flat_id = flat_id;
            visplane.light_level = lightlevel;
            visplane.fov_x = fov_x;
            self.active_indices.push(idx);
            return idx;
        }
        panic!("No more free visplanes");
    }

    pub fn render(&mut self, graphic: &Graphic, pixel_buf: &mut PixelBuf) {
        let visplanes = std::mem::take(&mut self.visplanes);
        for i in 0..self.active_indices.len() {
            let visplane = &visplanes[self.active_indices[i]];
            if visplane.fov_x.0 > visplane.fov_x.1 {
                continue;
            }
            if visplane.flat_id == graphic.sky_flat_id {
                self.render_sky(graphic, pixel_buf, visplane);
            } else {
                let flat_palettes = &graphic.flats[visplane.flat_id];
                for x in (visplane.fov_x.0..=(visplane.fov_x.1 + 1)).map(|x| x as usize) {
                    // 先頭に番兵があるので1加算する
                    let (mut t1, mut t2) = (visplane.top[x], visplane.top[x + 1]);
                    let (mut b1, mut b2) = (visplane.bottom[x], visplane.bottom[x + 1]);
                    // 上端で終了したspanが発生
                    while t1 < t2 && t1 <= b1 {
                        let y = t1 as usize;
                        let x_range = (self.span_start[y], x - 1);
                        self.render_plane(graphic, pixel_buf, flat_palettes, visplane, x_range, y);
                        t1 += 1;
                    }
                    // 下端で終了したspanが発生
                    while b1 > b2 && b1 >= t1 {
                        let y = b1 as usize;
                        let x_range = (self.span_start[y], x - 1);
                        self.render_plane(graphic, pixel_buf, flat_palettes, visplane, x_range, y);
                        b1 -= 1;
                    }
                    // 上端で新しいspanが発生
                    while t1 > t2 && t2 <= b2 {
                        self.span_start[t2 as usize] = x;
                        t2 += 1;
                    }
                    // 下端で新しいspanが発生
                    while b1 < b2 && b2 >= t2 {
                        self.span_start[b2 as usize] = x;
                        b2 -= 1;
                    }
                }
            }
        }
        self.visplanes = visplanes;
    }

    fn render_sky(&self, graphic: &Graphic, pixel_buf: &mut PixelBuf, visplane: &Visplane) {
        let texture = &graphic.wall_textures[graphic.sky_wall_texture_id];
        let palettes = &graphic.palettes[0];
        let x_range = (visplane.fov_x.0..=visplane.fov_x.1).map(|x| x as usize);
        for x in x_range {
            let y1 = visplane.top[x + 1];
            if y1 == 0xff {
                continue;
            }
            let angle = 4.0 * (self.player_angle + self.fov_x_to_angle[x]);
            let normal = angle.rem_euclid(360.0) / 360.0;
            let texture_x = (normal * texture.width as f32) as usize;
            let start_texture_y = 100.0 + (y1 as f32 - self.half_height) * self.sky_inverse_scale;
            let y_range = (y1..=visplane.bottom[x + 1])
                .map(|y| y as usize)
                .enumerate();
            for (i, y) in y_range {
                let texture_y = (start_texture_y + i as f32 * self.sky_inverse_scale) as isize;
                let wrapped_texture_y = wrap_texture_coord(texture_y, texture.height as isize);
                let idx = wrapped_texture_y * texture.width + texture_x;
                if let Some(pallete_idx) = texture.palettes[idx] {
                    let rgb = palettes[pallete_idx];
                    pixel_buf.set_pixel(x, y, rgb);
                }
            }
        }
    }

    pub fn render_plane(
        &mut self,
        graphic: &Graphic,
        pixel_buf: &mut PixelBuf,
        flat_palettes: &[usize],
        visplane: &Visplane,
        x_range: (usize, usize),
        y: usize,
    ) {
        let cache = &self.caches[y];
        let (dist, dir_x, dir_y) = if visplane.height != cache.height {
            let cache = &mut self.caches[y];
            cache.height = visplane.height;
            let dist = visplane.height * cache.y_slope;
            cache.dist = dist;
            // xを1進めた時のworld座標の変化量
            // 「プレイヤーの角度 - 90度」の方向へ移動する
            let angle = self.player_angle - 90.0;
            cache.dir_x = dist * angle.to_radians().cos() / self.half_width;
            cache.dir_y = dist * angle.to_radians().sin() / self.half_width;
            (dist, cache.dir_x, cache.dir_y)
        } else {
            (cache.dist, cache.dir_x, cache.dir_y)
        };
        // 斜め補正
        let length = dist * self.dist_scales[x_range.0];
        // x_range.0の角度
        let angle = self.player_angle + self.fov_x_to_angle[x_range.0];
        // プレイヤー座標からangleへlength分進んだ座標
        let begin_world_x = self.player_pos.x + length * angle.to_radians().cos();
        let begin_world_y = self.player_pos.y + length * angle.to_radians().sin();
        let color_idx = ((255 - visplane.light_level) / 16) as usize;
        let colormap = &graphic.colormaps[color_idx];
        let color_palettes = &graphic.palettes[0];
        let range = 0..(x_range.1 - x_range.0 + 1);
        for i in range.map(|i| i as f32) {
            // 現在のワールド座標
            let world_x = begin_world_x + dir_x * i;
            let world_y = begin_world_y + dir_y * i;
            // ワールド座標をテクスチャ座標に変換
            let texture_x = (world_x as i16 & FLAT_MASK) as usize;
            let texture_y = (world_y as i16 & FLAT_MASK) as usize;
            let palette_idx = flat_palettes[texture_y * FLAT_SIZE + texture_x];
            let mapped_idx = colormap[palette_idx];
            pixel_buf.set_pixel(x_range.0 + i as usize, y, color_palettes[mapped_idx]);
        }
    }
}

pub struct Visplane {
    pub height: f32,
    pub flat_id: usize,
    pub light_level: i16,
    pub fov_x: (i16, i16),
    // 配列の始点と終点に番兵を置くので2つ余分に確保
    pub top: [isize; SCREEN_WIDTH + 2],
    pub bottom: [isize; SCREEN_WIDTH + 2],
}

impl Visplane {
    pub fn initialize(&mut self) {
        self.height = 0.0;
        self.flat_id = 0;
        self.light_level = 0;
        self.fov_x = (-1, SCREEN_WIDTH as i16);
        self.top.fill(0xff);
        self.bottom.fill(0x00);
    }

    pub fn validte(&self, fov_x: (i16, i16)) -> bool {
        // 新しく追加された視野の範囲が全て書き込めるか調べる
        let inter_fov_x = (fov_x.0.max(self.fov_x.0), fov_x.1.min(self.fov_x.1));
        for x in inter_fov_x.0..=inter_fov_x.1 {
            if self.top[(x + 1) as usize] != 0xff {
                return false;
            }
        }
        true
    }

    pub fn set_y_range(&mut self, x: usize, y1: isize, y2: isize) {
        if y1 > y2 {
            return;
        }
        self.top[x + 1] = y1;
        self.bottom[x + 1] = y2;
    }
}

impl Default for Visplane {
    fn default() -> Self {
        Self {
            height: 0.0,
            flat_id: 0,
            light_level: 0,
            fov_x: (0, 0),
            top: [0xff; SCREEN_WIDTH + 2],
            bottom: [0xff; SCREEN_WIDTH + 2],
        }
    }
}

pub struct Cache {
    height: f32,
    y_slope: f32,
    dist: f32,
    dir_x: f32,
    dir_y: f32,
}

impl Cache {
    pub fn new(y_slope: f32) -> Self {
        Self {
            height: 0.0,
            y_slope,
            dist: 0.0,
            dir_x: 0.0,
            dir_y: 0.0,
        }
    }

    pub fn initialize(&mut self) {
        self.height = 0.0;
        self.dist = 0.0;
        self.dir_x = 0.0;
        self.dir_y = 0.0;
    }
}
