use unicode_width::UnicodeWidthChar;

use crate::color::Rgba;
use crate::element::Rect;
use crate::frame::{CellBuffer, CellStyle};

#[derive(Clone, Debug, Default)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub color: Option<Rgba>,
    pub fg_transparent: bool,
    pub cursor_anchor: bool,
    pub cursor_after: bool,
    pub bg: Option<Rgba>,
}

impl TextStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn cursor_anchor(mut self, after: bool) -> Self {
        self.cursor_anchor = true;
        self.cursor_after = after;
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

    pub fn color(mut self, color: Rgba) -> Self {
        self.color = Some(color);
        self.fg_transparent = false;
        self
    }

    pub fn color_transparent(mut self) -> Self {
        self.color = None;
        self.fg_transparent = true;
        self
    }

    pub fn bg(mut self, color: Rgba) -> Self {
        self.bg = Some(color);
        self
    }
}

#[derive(Clone, Debug)]
pub struct TextRun {
    pub text: String,
    pub style: TextStyle,
}

impl TextRun {
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::default(),
        }
    }

    pub fn styled(text: impl Into<String>, style: TextStyle) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StyledText {
    pub runs: Vec<TextRun>,
}

impl StyledText {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            runs: vec![TextRun::plain(text)],
        }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn push_run(mut self, text: impl Into<String>, style: TextStyle) -> Self {
        self.runs.push(TextRun::styled(text, style));
        self
    }

    pub fn push_plain(mut self, text: impl Into<String>) -> Self {
        self.runs.push(TextRun::plain(text));
        self
    }

    pub(crate) fn width_chars(&self) -> usize {
        let mut max_width = 0usize;
        let mut line_width = 0usize;

        for run in &self.runs {
            for ch in run.text.chars() {
                if ch == '\n' {
                    max_width = max_width.max(line_width);
                    line_width = 0;
                } else {
                    line_width =
                        line_width.saturating_add(UnicodeWidthChar::width(ch).unwrap_or(0));
                }
            }
        }

        max_width.max(line_width)
    }

    pub(crate) fn height_lines(&self) -> usize {
        let mut lines = 1usize;
        for run in &self.runs {
            lines = lines.saturating_add(run.text.chars().filter(|c| *c == '\n').count());
        }
        lines
    }

    pub(crate) fn wrapped_width_chars(&self, max_width: usize) -> usize {
        if max_width == 0 {
            return self.width_chars();
        }
        self.width_chars().min(max_width)
    }

    pub(crate) fn wrapped_height_lines(&self, max_width: usize) -> usize {
        if max_width == 0 {
            return self.height_lines();
        }

        let mut lines = 1usize;
        let mut line_width = 0usize;
        for run in &self.runs {
            for ch in run.text.chars() {
                if ch == '\n' {
                    lines = lines.saturating_add(1);
                    line_width = 0;
                    continue;
                }
                let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
                if line_width > 0 && line_width.saturating_add(ch_width) > max_width {
                    lines = lines.saturating_add(1);
                    line_width = 0;
                }
                line_width = line_width.saturating_add(ch_width);
            }
        }
        lines
    }

    pub(crate) fn render_at_clipped(
        &self,
        buffer: &mut CellBuffer,
        x: i32,
        y: i32,
        inherited_color: Option<Rgba>,
        clip: Rect,
    ) {
        let mut cursor_x = 0i32;
        let mut cursor_y = 0i32;
        let wrap_width = (clip.right - x).max(0);

        for run in &self.runs {
            let mut style = CellStyle::from(&run.style);
            if !style.fg_transparent {
                style.fg = style.fg.or(inherited_color);
            }

            for ch in run.text.chars() {
                if ch == '\n' {
                    cursor_y = cursor_y.saturating_add(1);
                    cursor_x = 0;
                    continue;
                }

                let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0) as i32;
                if wrap_width > 0 && cursor_x > 0 && cursor_x.saturating_add(ch_width) > wrap_width
                {
                    cursor_y = cursor_y.saturating_add(1);
                    cursor_x = 0;
                }
                let draw_x = x.saturating_add(cursor_x);
                let draw_y = y.saturating_add(cursor_y);
                if draw_x >= clip.left
                    && draw_x < clip.right
                    && draw_y >= clip.top
                    && draw_y < clip.bottom
                {
                    buffer.put_char(draw_x, draw_y, ch, style);
                }
                cursor_x = cursor_x.saturating_add(ch_width);
            }
        }
    }

}

pub fn styled_text(text: impl Into<String>) -> StyledText {
    StyledText::new(text)
}
