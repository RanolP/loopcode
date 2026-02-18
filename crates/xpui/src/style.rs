#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u32);

#[derive(Clone, Debug, Default)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub color: Option<Rgb>,
    pub bg: Option<Rgb>,
}

impl TextStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    pub fn color(mut self, color: Rgb) -> Self {
        self.color = Some(color);
        self
    }

    pub fn bg(mut self, color: Rgb) -> Self {
        self.bg = Some(color);
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct BoxStyle {
    pub bg: Option<Rgb>,
    pub text_color: Option<Rgb>,
}

impl BoxStyle {
    pub fn bg(mut self, color: Rgb) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn text_color(mut self, color: Rgb) -> Self {
        self.text_color = Some(color);
        self
    }
}

pub fn rgb(hex: u32) -> Rgb {
    Rgb(hex)
}
