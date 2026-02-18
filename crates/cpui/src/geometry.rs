#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pixels(pub f32);

#[derive(Clone, Copy, Debug, Default)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Bounds {
    pub origin: Point<Pixels>,
    pub size: Size<Pixels>,
}

pub fn px(value: f32) -> Pixels {
    Pixels(value)
}

pub fn size(width: Pixels, height: Pixels) -> Size<Pixels> {
    Size { width, height }
}
