mod app;
mod color;
mod context;
mod element;
mod entity;
mod geometry;
mod text;
mod view;
mod window;

pub use app::{App, Application, InputEvent, KeyInput, Result, SharedString};
pub use color::{Rgba, black, blue, green, red, rgb, white, yellow};
pub use context::{
    AppContext, Context, EventEmitter, Focusable, Global, GpuiBorrow, Reservation, VisualContext,
};
pub use element::{AnyElement, Div, IntoElement, ScrollView, div, scroll_view};
pub use entity::{AnyEntity, AnyView, Entity, EntityId, WeakEntity, WindowId};
pub use geometry::{Bounds, Pixels, Point, Size, px, size};
pub use text::{StyledText, TextRun, TextStyle, styled_text};
pub use view::Render;
pub use window::{
    AnyWindowHandle, TitlebarOptions, Window, WindowBackgroundAppearance, WindowBounds,
    WindowDecorations, WindowHandle, WindowKind, WindowOptions,
};

pub mod prelude {
    pub use crate::{IntoElement, Render};
}
