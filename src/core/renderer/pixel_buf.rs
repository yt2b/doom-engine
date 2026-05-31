pub struct PixelBuf {
    pub width: usize,
    pub height: usize,
    pub width_step: usize,
    pub buf: Vec<u8>,
}

impl PixelBuf {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            width_step: width * 4,
            buf: vec![0; width * height * 4],
        }
    }

    pub fn clear(&mut self) {
        // アルファチャネル以外を0にする
        for i in 0..self.buf.len() {
            self.buf[i] = if i % 4 == 3 { 255 } else { 0 };
        }
    }

    pub fn set_pixel(&mut self, idx: usize, rgb: (u8, u8, u8)) {
        if idx >= self.buf.len() {
            return;
        }
        unsafe {
            let ptr = self.buf.as_mut_ptr().add(idx);
            ptr.write(rgb.0); // r
            ptr.add(1).write(rgb.1); // g
            ptr.add(2).write(rgb.2); // b
        }
    }
}
