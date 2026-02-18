#[cfg(feature = "backend-cpui")]
mod cpui;
mod gpui;

#[cfg(feature = "backend-cpui")]
pub use cpui::CpuiBackend;
#[cfg(feature = "backend-cpui")]
pub(crate) use cpui::run_cpui;
pub(crate) use gpui::run_gpui;
pub use gpui::{GpuiAdapter, GpuiBackend};
