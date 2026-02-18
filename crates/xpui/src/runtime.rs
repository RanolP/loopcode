use crate::Node;

pub trait UiApp {
    fn render(&mut self) -> Node;

    fn on_input(&mut self, _event: UiInputEvent) {}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiKeyInput {
    Tab,
    BackTab,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Esc,
    Char(char),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiInputEvent {
    Key(UiKeyInput),
    ScrollLines(i16),
}

#[derive(Clone, Copy, Debug)]
pub struct WindowSize {
    pub width: f32,
    pub height: f32,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: 80.0,
            height: 24.0,
        }
    }
}

#[cfg(feature = "backend-cpui")]
pub fn run_cpui<A: UiApp + 'static>(app: A) {
    crate::backends::run_cpui(app, WindowSize::default())
}

#[cfg(feature = "backend-cpui")]
pub fn run_cpui_with_size<A: UiApp + 'static>(app: A, size: WindowSize) {
    crate::backends::run_cpui(app, size)
}

pub fn run_gpui<A: UiApp + 'static>(app: A) {
    crate::backends::run_gpui(app, WindowSize::default())
}

pub fn run_gpui_with_size<A: UiApp + 'static>(app: A, size: WindowSize) {
    crate::backends::run_gpui(app, size)
}
