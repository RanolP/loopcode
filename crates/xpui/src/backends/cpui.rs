use cpui::IntoElement;

use crate::{
    backend::Backend,
    node::{Axis, Node, RichText},
    runtime::{UiApp, WindowSize},
    style::{Rgb, TextStyle},
};

pub struct CpuiBackend;

impl Backend for CpuiBackend {
    type Output = cpui::AnyElement;

    fn render_node(&mut self, node: Node) -> Self::Output {
        node_to_cpui(node)
    }
}

pub(crate) fn run_cpui<A: UiApp + 'static>(app: A, size: WindowSize) {
    struct Host<A> {
        app: A,
    }

    impl<A: UiApp + 'static> cpui::Render for Host<A> {
        fn render(
            &mut self,
            _window: &mut cpui::Window,
            _cx: &mut cpui::Context<'_, Self>,
        ) -> impl cpui::IntoElement {
            let mut backend = CpuiBackend;
            crate::render(&mut backend, self.app.render())
        }
    }

    cpui::Application::new().run(move |cx: &mut cpui::App| {
        let bounds = cpui::Bounds::centered(
            None,
            cpui::size(cpui::px(size.width), cpui::px(size.height)),
            cx,
        );

        let _ = cx.open_window(
            cpui::WindowOptions {
                window_bounds: Some(cpui::WindowBounds::Windowed(bounds)),
                ..cpui::WindowOptions::default()
            },
            |_window, cx| cx.new(|_cx| Host { app }),
        );

        cx.activate(true);
    });
}

fn node_to_cpui(node: Node) -> cpui::AnyElement {
    match node {
        Node::Empty => cpui::AnyElement::Empty,
        Node::RichText(text) => text_to_cpui(text).into_any_element(),
        Node::Container(container) => {
            let mut out = cpui::div();
            if let Some(bg) = container.style.bg {
                out = out.bg(to_cpui_color(bg));
            }
            if let Some(text_color) = container.style.text_color {
                out = out.text_color(to_cpui_color(text_color));
            }
            out.child(node_to_cpui(*container.child)).into_any_element()
        }
        Node::Stack(stack) => {
            let mut out = cpui::div().flex();

            if matches!(stack.axis, Axis::Column) {
                out = out.flex_col();
            }
            if stack.justify_center {
                out = out.justify_center();
            }
            if stack.items_center {
                out = out.items_center();
            }

            out = match stack.gap {
                0 => out,
                1..=2 => out.gap_2(),
                _ => out.gap_3(),
            };

            for child in stack.children {
                out = out.child(node_to_cpui(child));
            }

            out.into_any_element()
        }
    }
}

fn text_to_cpui(text: RichText) -> cpui::StyledText {
    if text.runs.is_empty() {
        return cpui::StyledText::new("");
    }

    let mut out = cpui::StyledText::empty();
    for run in text.runs {
        out = out.push_run(run.text, to_cpui_text_style(run.style));
    }
    out
}

fn to_cpui_text_style(style: TextStyle) -> cpui::TextStyle {
    let mut out = cpui::TextStyle::new();
    if style.bold {
        out = out.bold();
    }
    if style.italic {
        out = out.italic();
    }
    if style.underline {
        out = out.underline();
    }
    if style.strikethrough {
        out = out.strikethrough();
    }
    if let Some(color) = style.color {
        out = out.color(to_cpui_color(color));
    }
    if let Some(bg) = style.bg {
        out = out.bg(to_cpui_color(bg));
    }
    out
}

fn to_cpui_color(color: Rgb) -> cpui::Rgba {
    cpui::rgb(color.0)
}
