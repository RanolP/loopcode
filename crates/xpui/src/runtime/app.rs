use crate::Node;

use super::{FocusEntry, FocusState, UiInputEvent, WindowSize};

pub trait UiApp {
    fn render(&mut self) -> Node;

    fn on_input(&mut self, _event: UiInputEvent) {}

    fn set_window_size(&mut self, _size: WindowSize) {}

    fn focus_state(&mut self) -> Option<&mut FocusState> {
        None
    }

    fn on_focus_entries(&mut self, _entries: &[FocusEntry]) {}
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
