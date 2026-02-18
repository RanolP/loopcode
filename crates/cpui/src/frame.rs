use unicode_width::UnicodeWidthChar;

use crate::{color::Rgba, text::TextStyle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Glyph {
    Char(char),
    WideTail,
}

impl Default for Glyph {
    fn default() -> Self {
        Self::Char(' ')
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CellStyle {
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) underline: bool,
    pub(crate) strikethrough: bool,
    pub(crate) fg: Option<Rgba>,
    pub(crate) cursor_anchor: bool,
    pub(crate) cursor_after: bool,
    pub(crate) bg: Option<Rgba>,
}

impl From<TextStyle> for CellStyle {
    fn from(value: TextStyle) -> Self {
        Self {
            bold: value.bold,
            italic: value.italic,
            underline: value.underline,
            strikethrough: value.strikethrough,
            fg: value.color,
            cursor_anchor: value.cursor_anchor,
            cursor_after: value.cursor_after,
            bg: value.bg,
        }
    }
}

impl From<&TextStyle> for CellStyle {
    fn from(value: &TextStyle) -> Self {
        Self {
            bold: value.bold,
            italic: value.italic,
            underline: value.underline,
            strikethrough: value.strikethrough,
            fg: value.color,
            cursor_anchor: value.cursor_anchor,
            cursor_after: value.cursor_after,
            bg: value.bg,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Cell {
    pub(crate) glyph: Glyph,
    pub(crate) style: CellStyle,
}

impl Cell {
    pub(crate) const fn blank() -> Self {
        Self {
            glyph: Glyph::Char(' '),
            style: CellStyle {
                bold: false,
                italic: false,
                underline: false,
                strikethrough: false,
                fg: None,
                cursor_anchor: false,
                cursor_after: false,
                bg: None,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CellBuffer {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
    cursor: Option<(u16, u16)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CellRun {
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) text: String,
    pub(crate) style: CellStyle,
}

impl CellBuffer {
    pub(crate) fn new(width: u16, height: u16) -> Self {
        let len = usize::from(width) * usize::from(height);
        Self {
            width,
            height,
            cells: vec![Cell::blank(); len],
            cursor: None,
        }
    }

    pub(crate) fn width(&self) -> u16 {
        self.width
    }

    pub(crate) fn height(&self) -> u16 {
        self.height
    }

    pub(crate) fn get(&self, x: u16, y: u16) -> Cell {
        self.cells[self.idx(x, y)]
    }

    pub(crate) fn set(&mut self, x: u16, y: u16, cell: Cell) {
        let idx = self.idx(x, y);
        self.cells[idx] = cell;
    }

    pub(crate) fn set_bg(&mut self, x: u16, y: u16, bg: Rgba) {
        let mut cell = self.get(x, y);
        cell.style.bg = Some(bg);
        self.set(x, y, cell);
    }

    pub(crate) fn put_char(&mut self, x: i32, y: i32, ch: char, style: CellStyle) {
        if x < 0 || y < 0 {
            return;
        }
        let Ok(x) = u16::try_from(x) else {
            return;
        };
        let Ok(y) = u16::try_from(y) else {
            return;
        };
        if x >= self.width || y >= self.height {
            return;
        }

        let glyph_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if glyph_width == 0 {
            return;
        }

        let mut head_style = style;
        if head_style.bg.is_none() {
            head_style.bg = self.get(x, y).style.bg;
        }
        self.set(
            x,
            y,
            Cell {
                glyph: Glyph::Char(ch),
                style: head_style,
            },
        );
        if style.cursor_anchor {
            let advance = if style.cursor_after {
                glyph_width as u16
            } else {
                0
            };
            self.set_cursor(x, y, advance);
        }

        if glyph_width > 1 {
            let tail_x = x.saturating_add(1);
            if tail_x < self.width {
                let mut tail_style = style;
                if tail_style.bg.is_none() {
                    tail_style.bg = self.get(tail_x, y).style.bg;
                }
                self.set(
                    tail_x,
                    y,
                    Cell {
                        glyph: Glyph::WideTail,
                        style: tail_style,
                    },
                );
            }
        }
    }

    pub(crate) fn cursor(&self) -> Option<(u16, u16)> {
        self.cursor
    }

    pub(crate) fn diff_runs(&self, prev: &Self) -> Vec<CellRun> {
        if self.width != prev.width || self.height != prev.height {
            return self.full_runs();
        }

        let mut runs = Vec::new();
        for y in 0..self.height {
            let mut x = 0u16;
            while x < self.width {
                let current = self.get(x, y);
                let previous = prev.get(x, y);
                if !should_emit(previous, current) {
                    x = x.saturating_add(1);
                    continue;
                }

                let run_x = x;
                let run_style = current.style;
                let mut text = String::new();
                while x < self.width {
                    let curr = self.get(x, y);
                    let prev = prev.get(x, y);
                    if !should_emit(prev, curr) || curr.style != run_style {
                        break;
                    }
                    if let Glyph::Char(ch) = curr.glyph {
                        text.push(ch);
                    }
                    x = x.saturating_add(1);
                }
                if !text.is_empty() {
                    runs.push(CellRun {
                        x: run_x,
                        y,
                        text,
                        style: run_style,
                    });
                }
            }
        }
        runs
    }

    fn full_runs(&self) -> Vec<CellRun> {
        let empty = CellBuffer::new(self.width, self.height);
        self.diff_runs(&empty)
    }

    fn idx(&self, x: u16, y: u16) -> usize {
        usize::from(y) * usize::from(self.width) + usize::from(x)
    }

    fn set_cursor(&mut self, x: u16, y: u16, advance: u16) {
        let x = x.saturating_add(advance);
        let x = x.min(self.width.saturating_sub(1));
        let y = y.min(self.height.saturating_sub(1));
        self.cursor = Some((x, y));
    }
}

fn should_emit(previous: Cell, current: Cell) -> bool {
    previous != current && !matches!(current.glyph, Glyph::WideTail)
}
