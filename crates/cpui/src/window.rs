use std::{io, marker::PhantomData};

use crossterm::{
    cursor, execute,
    terminal::{self, Clear, ClearType},
};

use crate::{
    element::AnyElement,
    entity::WindowId,
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
}

impl Window {
    pub(crate) fn new(id: WindowId, options: WindowOptions) -> Self {
        Self { id, options }
    }

    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn terminal_size(&self) -> io::Result<(u16, u16)> {
        terminal::size()
    }

    pub(crate) fn draw(&mut self, element: &AnyElement) -> io::Result<()> {
        if !crate::app::is_alt_screen_active() {
            return Ok(());
        }
        let mut out = io::stdout();
        execute!(out, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        let (w, h) = terminal::size()?;
        crate::element::render_element(&mut out, element, w, h)
    }
}
