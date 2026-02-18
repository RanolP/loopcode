mod backend;
mod backends;
mod node;
mod runtime;
mod style;
mod widgets;

pub use backend::{Backend, render};
#[cfg(feature = "backend-cpui")]
pub use backends::CpuiBackend;
pub use backends::{GpuiAdapter, GpuiBackend};
pub use node::{Axis, FocusId, IntoNode, Node, RichText, ScrollView, TextInput, TextRun};
pub use runtime::{
    FocusListState, FocusState, TextInputState, UiApp, UiInputEvent, UiKeyInput, WindowSize,
    run_gpui, run_gpui_with_size,
};
#[cfg(feature = "backend-cpui")]
pub use runtime::{run_cpui, run_cpui_with_size};
pub use style::{BoxStyle, Rgb, TextStyle, rgb};
pub use widgets::{
    ContainerWidget, ScrollViewWidget, StackWidget, TextInputWidget, TextWidget, column, container,
    row, scroll_view, text, text_input, text_input_from_state,
};
