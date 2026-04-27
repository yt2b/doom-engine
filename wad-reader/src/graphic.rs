pub struct Graphic {
    pub palettes: Vec<Vec<(u8, u8, u8)>>,
}

impl Graphic {
    pub fn new(palettes: Vec<Vec<(u8, u8, u8)>>) -> Self {
        Self { palettes }
    }
}

pub fn get_palettes(bytes: &[u8]) -> Vec<Vec<(u8, u8, u8)>> {
    // 768バイトごとに区切って、さらに3バイトごとにRGB値を読み取る
    bytes
        .chunks(768)
        .map(|chunk| {
            chunk
                .chunks(3)
                .map(|rgb| (rgb[0], rgb[1], rgb[2]))
                .collect()
        })
        .collect()
}
