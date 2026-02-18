use crate::FocusId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiKeyInput {
    Tab,
    BackTab,
    Left,
    Right,
    WordLeft,
    WordRight,
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Backspace,
    BackspaceWord,
    Delete,
    Enter,
    Submit,
    Esc,
    Interrupt,
    Char(char),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiInputEvent {
    Key(UiKeyInput),
    ScrollLines(i16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusNavOutcome {
    Ignored,
    Handled,
    RequestQuit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusKind {
    Generic,
    TextInput,
    ScrollRegion,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FocusPath(pub Vec<usize>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FocusEntry {
    pub id: FocusId,
    pub path: FocusPath,
    pub kind: FocusKind,
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
