pub struct PixelBuf {
    pub width: usize,
    pub height: usize,
    pub buf: Vec<u8>,
}

impl PixelBuf {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buf: vec![0; width * height * 4],
        }
    }

    pub fn clear(&mut self) {
        // アルファチャネル以外を0にする
        for i in 0..self.buf.len() {
            self.buf[i] = if i % 4 == 3 { 255 } else { 0 };
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        let i = y * self.width * 4 + x * 4;
        self.buf[i] = rgb.0; // r
        self.buf[i + 1] = rgb.1; // g
        self.buf[i + 2] = rgb.2; // b
    }
}
