use crate::{
    backend::Backend,
    node::{Axis, Node, RichText},
    runtime::{UiApp, WindowSize},
};

pub trait GpuiAdapter {
    type Output;

    fn render_node(&mut self, node: Node) -> Self::Output;
}

pub struct GpuiBackend<A> {
    adapter: A,
}

impl<A> GpuiBackend<A> {
    pub fn new(adapter: A) -> Self {
        Self { adapter }
    }
}

impl<A> Backend for GpuiBackend<A>
where
    A: GpuiAdapter,
{
    type Output = A::Output;

    fn render_node(&mut self, node: Node) -> Self::Output {
        self.adapter.render_node(node)
    }
}

#[cfg(feature = "backend-gpui")]
pub(crate) fn run_gpui<A: UiApp + 'static>(app: A, _size: WindowSize) {
    use gpui::{App, AppContext, Application, Context, IntoElement, Render, Window, WindowOptions};

    struct Host<A> {
        app: A,
    }

    impl<A: UiApp + 'static> Render for Host<A> {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            root_to_gpui(self.app.render())
        }
    }

    Application::new().run(move |cx: &mut App| {
        let _ = cx.open_window(WindowOptions::default(), |_window, cx| {
            cx.new(|_cx| Host { app })
        });
        cx.activate(true);
    });
}

#[cfg(not(feature = "backend-gpui"))]
pub(crate) fn run_gpui<A: UiApp + 'static>(app: A, _size: WindowSize) {
    let _ = app;
    panic!("xpui built without backend-gpui feature");
}

#[cfg(feature = "backend-gpui")]
fn root_to_gpui(node: Node) -> gpui::AnyElement {
    use gpui::{IntoElement, ParentElement, Styled, div};

    match node {
        Node::Container(container) => {
            let mut root = div().size_full().font_family("DejaVu Sans");
            if let Some(bg) = container.style.bg {
                root = root.bg(gpui::rgb(bg.0));
            }
            if let Some(text_color) = container.style.text_color {
                root = root.text_color(gpui::rgb(text_color.0));
            }
            root.child(node_to_gpui(*container.child))
                .into_any_element()
        }
        other => div()
            .size_full()
            .font_family("DejaVu Sans")
            .child(node_to_gpui(other))
            .into_any_element(),
    }
}

#[cfg(feature = "backend-gpui")]
fn node_to_gpui(node: Node) -> gpui::AnyElement {
    use gpui::{IntoElement, ParentElement, Styled, div};

    match node {
        Node::Empty => div().into_any_element(),
        Node::RichText(text) => rich_text_to_gpui(text).into_any_element(),
        Node::Container(container) => {
            let mut out = div();
            if let Some(bg) = container.style.bg {
                out = out.bg(gpui::rgb(bg.0));
            }
            if let Some(text_color) = container.style.text_color {
                out = out.text_color(gpui::rgb(text_color.0));
            }
            out.child(node_to_gpui(*container.child)).into_any_element()
        }
        Node::Stack(stack) => {
            let mut out = div().flex();
            if matches!(stack.axis, Axis::Column) {
                out = out.flex_col();
            }
            if stack.justify_center {
                out = out.justify_center();
            }
            if stack.items_center {
                out = out.items_center();
            }
            for child in stack.children {
                out = out.child(node_to_gpui(child));
            }
            out.into_any_element()
        }
    }
}

#[cfg(feature = "backend-gpui")]
fn rich_text_to_gpui(text: RichText) -> gpui::StyledText {
    use gpui::{
        FontStyle, FontWeight, HighlightStyle, StrikethroughStyle, StyledText, UnderlineStyle, px,
    };

    let mut full = String::new();
    let mut highlights = Vec::new();

    for run in text.runs {
        let start = full.len();
        full.push_str(&run.text);
        let end = full.len();

        let mut style = HighlightStyle::default();
        let mut changed = false;

        if let Some(color) = run.style.color {
            style.color = Some(gpui::rgb(color.0).into());
            changed = true;
        }
        if let Some(bg) = run.style.bg {
            style.background_color = Some(gpui::rgb(bg.0).into());
            changed = true;
        }
        if run.style.bold {
            style.font_weight = Some(FontWeight::BOLD);
            changed = true;
        }
        if run.style.italic {
            style.font_style = Some(FontStyle::Italic);
            changed = true;
        }
        if run.style.underline {
            style.underline = Some(UnderlineStyle {
                thickness: px(1.0),
                color: None,
                wavy: false,
            });
            changed = true;
        }
        if run.style.strikethrough {
            style.strikethrough = Some(StrikethroughStyle {
                thickness: px(1.0),
                color: None,
            });
            changed = true;
        }

        if changed && start < end {
            highlights.push((start..end, style));
        }
    }

    if highlights.is_empty() {
        StyledText::new(full)
    } else {
        StyledText::new(full).with_highlights(highlights)
    }
}
