#[derive(Clone, Copy, Debug, Default)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub fn rgb(hex: u32) -> Rgba {
    Rgba {
        r: ((hex >> 16) & 0xff) as u8,
        g: ((hex >> 8) & 0xff) as u8,
        b: (hex & 0xff) as u8,
    }
}

pub fn red() -> Rgba {
    rgb(0xff0000)
}

pub fn green() -> Rgba {
    rgb(0x00ff00)
}

pub fn blue() -> Rgba {
    rgb(0x0000ff)
}

pub fn yellow() -> Rgba {
    rgb(0xffff00)
}

pub fn black() -> Rgba {
    rgb(0x000000)
}

pub fn white() -> Rgba {
    rgb(0xffffff)
}
