use std::{
    io::{self, BufWriter, Write},
    marker::PhantomData,
    time::{Duration, Instant},
};

use crossterm::{
    cursor,
    style::{
        Attribute, Color as TermColor, Print, ResetColor, SetAttribute, SetBackgroundColor,
        SetForegroundColor,
    },
    terminal::{self, BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate},
};

use crate::{
    element::AnyElement,
    entity::WindowId,
    frame::{CellBuffer, CellStyle},
    geometry::{Bounds, Pixels, Size},
};

#[derive(Clone, Copy, Debug)]
pub struct AnyWindowHandle {
    pub(crate) id: WindowId,
}

#[derive(Clone, Copy, Debug)]
pub struct WindowHandle<T> {
    pub(crate) id: WindowId,
    pub(crate) _marker: PhantomData<T>,
}

impl<T> WindowHandle<T> {
    pub(crate) fn new(id: WindowId) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub fn to_any(self) -> AnyWindowHandle {
        AnyWindowHandle { id: self.id }
    }
}

#[derive(Clone, Debug)]
pub enum WindowBounds {
    Windowed(Bounds),
}

#[derive(Clone, Debug, Default)]
pub struct TitlebarOptions;

#[derive(Clone, Debug, Default)]
pub enum WindowKind {
    #[default]
    Normal,
}

#[derive(Clone, Debug, Default)]
pub enum WindowBackgroundAppearance {
    #[default]
    Opaque,
}

#[derive(Clone, Debug)]
pub enum WindowDecorations {
    Server,
}

#[derive(Clone, Debug)]
pub struct WindowOptions {
    pub window_bounds: Option<WindowBounds>,
    pub titlebar: Option<TitlebarOptions>,
    pub focus: bool,
    pub show: bool,
    pub kind: WindowKind,
    pub is_movable: bool,
    pub is_resizable: bool,
    pub is_minimizable: bool,
    pub display_id: Option<u64>,
    pub window_background: WindowBackgroundAppearance,
    pub app_id: Option<String>,
    pub window_min_size: Option<Size<Pixels>>,
    pub window_decorations: Option<WindowDecorations>,
    pub tabbing_identifier: Option<String>,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            window_bounds: None,
            titlebar: None,
            focus: true,
            show: true,
            kind: WindowKind::Normal,
            is_movable: true,
            is_resizable: true,
            is_minimizable: true,
            display_id: None,
            window_background: WindowBackgroundAppearance::Opaque,
            app_id: None,
            window_min_size: None,
            window_decorations: None,
            tabbing_identifier: None,
        }
    }
}

pub struct Window {
    id: WindowId,
    pub options: WindowOptions,
    prev_frame: Option<CellBuffer>,
    cursor_visible: bool,
    cursor_blink_at: Instant,
    terminal_focused: bool,
}

impl Window {
    pub(crate) fn new(id: WindowId, options: WindowOptions) -> Self {
        Self {
            id,
            options,
            prev_frame: None,
            cursor_visible: true,
            cursor_blink_at: Instant::now(),
            terminal_focused: true,
        }
    }

    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn terminal_size(&self) -> io::Result<(u16, u16)> {
        terminal::size()
    }

    pub(crate) fn draw(&mut self, element: &AnyElement) -> io::Result<()> {
        let stdout = io::stdout();
        let mut out = BufWriter::new(stdout.lock());
        crossterm::queue!(out, BeginSynchronizedUpdate)?;
        let (w, h) = terminal::size()?;
        let current = crate::element::render_element(element, w, h)?;
        let mut resized = false;
        let prev = self
            .prev_frame
            .take()
            .inspect(|frame| {
                resized = frame.width() != w || frame.height() != h;
            })
            .filter(|frame| frame.width() == w && frame.height() == h)
            .unwrap_or_else(|| CellBuffer::new(w, h));
        if resized {
            crossterm::queue!(out, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        }
        flush_diff(&mut out, &prev, &current)?;
        if self.terminal_focused {
            if let Some((cx, cy)) = current.cursor() {
                if self.cursor_blink_at.elapsed() >= Duration::from_millis(570) {
                    self.cursor_visible = !self.cursor_visible;
                    self.cursor_blink_at = Instant::now();
                }
                if self.cursor_visible {
                    crossterm::queue!(out, cursor::MoveTo(cx, cy), cursor::Show)?;
                } else {
                    crossterm::queue!(out, cursor::Hide)?;
                }
            } else {
                self.cursor_visible = true;
                self.cursor_blink_at = Instant::now();
                crossterm::queue!(out, cursor::Hide)?;
            }
        } else {
            if self.cursor_blink_at.elapsed() >= Duration::from_millis(570) {
                self.cursor_visible = true;
                self.cursor_blink_at = Instant::now();
            }
            crossterm::queue!(out, cursor::Hide)?;
        }
        self.prev_frame = Some(current);
        crossterm::queue!(out, EndSynchronizedUpdate)?;
        out.flush()
    }

    pub(crate) fn note_input_activity(&mut self) {
        self.cursor_visible = true;
        self.cursor_blink_at = Instant::now();
    }

    pub(crate) fn set_terminal_focus(&mut self, focused: bool) {
        self.terminal_focused = focused;
        self.cursor_visible = true;
        self.cursor_blink_at = Instant::now();
    }
}

fn flush_diff(out: &mut impl io::Write, prev: &CellBuffer, current: &CellBuffer) -> io::Result<()> {
    let mut style_emitter = StyleEmitter::default();
    for run in current.diff_runs(prev) {
        style_emitter.apply(out, run.style)?;
        crossterm::queue!(out, cursor::MoveTo(run.x, run.y), Print(run.text))?;
    }

    style_emitter.reset(out)
}


#[derive(Default)]
struct StyleEmitter {
    current: CellStyle,
}

impl StyleEmitter {
    fn apply(&mut self, out: &mut impl io::Write, target: CellStyle) -> io::Result<()> {
        if self.current == target {
            return Ok(());
        }

        if self.current.fg != target.fg {
            if let Some(color) = target.fg {
                crossterm::queue!(
                    out,
                    SetForegroundColor(TermColor::Rgb {
                        r: color.r,
                        g: color.g,
                        b: color.b,
                    })
                )?;
            } else {
                crossterm::queue!(out, SetForegroundColor(TermColor::Reset))?;
            }
        }

        if self.current.bg != target.bg {
            if let Some(bg) = target.bg {
                crossterm::queue!(
                    out,
                    SetBackgroundColor(TermColor::Rgb {
                        r: bg.r,
                        g: bg.g,
                        b: bg.b,
                    })
                )?;
            } else {
                crossterm::queue!(out, SetBackgroundColor(TermColor::Reset))?;
            }
        }

        let attrs_changed = self.current.bold != target.bold
            || self.current.italic != target.italic
            || self.current.underline != target.underline
            || self.current.strikethrough != target.strikethrough;

        if attrs_changed {
            crossterm::queue!(out, SetAttribute(Attribute::Reset))?;
            if target.bold {
                crossterm::queue!(out, SetAttribute(Attribute::Bold))?;
            }
            if target.italic {
                crossterm::queue!(out, SetAttribute(Attribute::Italic))?;
            }
            if target.underline {
                crossterm::queue!(out, SetAttribute(Attribute::Underlined))?;
            }
            if target.strikethrough {
                crossterm::queue!(out, SetAttribute(Attribute::CrossedOut))?;
            }
        }

        self.current = target;
        Ok(())
    }

    fn reset(&mut self, out: &mut impl io::Write) -> io::Result<()> {
        if self.current != CellStyle::default() {
            self.current = CellStyle::default();
            crossterm::queue!(out, SetAttribute(Attribute::Reset), ResetColor)?;
        }
        Ok(())
    }
}
