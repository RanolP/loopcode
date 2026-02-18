use crate::{
    backend::Backend,
    node::{Axis, Node, RichText},
    runtime::{FocusEntry, FocusNavOutcome, UiApp, UiInputEvent, UiKeyInput, WindowSize},
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
        focus_order: Vec<FocusEntry>,
        root_focus: gpui::FocusHandle,
        wheel_line_carry: f32,
        window_size: WindowSize,
    }

    impl<A: UiApp + 'static> Render for Host<A> {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            use gpui::{InteractiveElement, ParentElement, Styled, div, px};

            window.focus(&self.root_focus);
            self.app.set_window_size(self.window_size);

            let node = self.app.render();
            let mut focus_order = Vec::new();
            node.collect_focus_entries(&mut focus_order);
            self.focus_order = focus_order.clone();
            if let Some(focus) = self.app.focus_state() {
                focus.ensure_valid(&focus_order);
            }
            self.app.on_focus_entries(&focus_order);

            let mut root = div()
                .size_full()
                .font_family("DejaVu Sans")
                .track_focus(&self.root_focus)
                .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                    let mapped = if event.keystroke.key == "tab" {
                        Some(if event.keystroke.modifiers.shift {
                            UiKeyInput::ShiftTab
                        } else {
                            UiKeyInput::Tab
                        })
                    } else {
                        map_gpui_key_event(event)
                    };

                    let Some(mapped) = mapped else {
                        return;
                    };
                    let ui_event = UiInputEvent::Key(mapped);

                    let nav_outcome = if let Some(focus) = this.app.focus_state() {
                        focus.handle_navigation(ui_event, &this.focus_order)
                    } else {
                        FocusNavOutcome::Ignored
                    };

                    match nav_outcome {
                        FocusNavOutcome::Ignored => this.app.on_input(ui_event),
                        FocusNavOutcome::Handled => {}
                        FocusNavOutcome::RequestQuit => cx.quit(),
                    }
                    cx.notify();
                    window.refresh();
                }))
                .on_scroll_wheel(cx.listener(
                    |this, event: &gpui::ScrollWheelEvent, window, cx| {
                        let delta_lines = match event.delta {
                            gpui::ScrollDelta::Lines(delta) => delta.y,
                            gpui::ScrollDelta::Pixels(delta) => delta.y / px(18.0),
                        };

                        this.wheel_line_carry += delta_lines;
                        let whole_lines = this.wheel_line_carry.trunc() as i16;
                        this.wheel_line_carry -= whole_lines as f32;

                        if whole_lines != 0 {
                            this.app.on_input(UiInputEvent::ScrollLines(whole_lines));
                            cx.notify();
                            window.refresh();
                        }
                    },
                ));

            match node {
                Node::Container(container) => {
                    if let Some(bg) = container.style.bg {
                        root = root.bg(gpui::rgb(bg.0));
                    }
                    if let Some(text_color) = container.style.text_color {
                        root = root.text_color(gpui::rgb(text_color.0));
                    }
                    root.child(node_to_gpui(
                        *container.child,
                        self.window_size.width.max(1.0) as usize,
                    ))
                    .into_any_element()
                }
                other => root
                    .child(node_to_gpui(
                        other,
                        self.window_size.width.max(1.0) as usize,
                    ))
                    .into_any_element(),
            }
        }
    }

    Application::new().run(move |cx: &mut App| {
        let _ = cx.open_window(WindowOptions::default(), |_window, cx| {
            cx.new(|cx| Host {
                app,
                focus_order: Vec::new(),
                root_focus: cx.focus_handle(),
                wheel_line_carry: 0.0,
                window_size: _size,
            })
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
fn map_gpui_key_event(event: &gpui::KeyDownEvent) -> Option<UiKeyInput> {
    let secondary = event.keystroke.modifiers.secondary();
    let alt = event.keystroke.modifiers.alt;
    let shift = event.keystroke.modifiers.shift;
    let is_submit = alt && matches!(event.keystroke.key.as_str(), "enter" | "return");
    if is_submit {
        return Some(UiKeyInput::Submit);
    }
    if shift && event.keystroke.key == "tab" {
        return Some(UiKeyInput::ShiftTab);
    }
    if secondary && matches!(event.keystroke.key_char.as_deref(), Some("w")) {
        return Some(UiKeyInput::BackspaceWord);
    }
    match event.keystroke.key.as_str() {
        "left" if secondary => Some(UiKeyInput::WordLeft),
        "right" if secondary => Some(UiKeyInput::WordRight),
        "backspace" if secondary => Some(UiKeyInput::BackspaceWord),
        "left" => Some(UiKeyInput::Left),
        "right" => Some(UiKeyInput::Right),
        "up" => Some(UiKeyInput::Up),
        "down" => Some(UiKeyInput::Down),
        "home" => Some(UiKeyInput::Home),
        "end" => Some(UiKeyInput::End),
        "backspace" => Some(UiKeyInput::Backspace),
        "delete" => Some(UiKeyInput::Delete),
        "enter" => Some(UiKeyInput::Enter),
        "escape" => Some(UiKeyInput::Esc),
        _ => {
            let text = event
                .keystroke
                .key_char
                .as_deref()
                .unwrap_or(event.keystroke.key.as_str());
            if text.chars().count() == 1 {
                text.chars().next().map(UiKeyInput::Char)
            } else {
                None
            }
        }
    }
}

#[cfg(feature = "backend-gpui")]
fn node_to_gpui(node: Node, viewport_columns: usize) -> gpui::AnyElement {
    use gpui::{IntoElement, ParentElement, Styled, div};

    match node {
        Node::Empty => div().into_any_element(),
        Node::RichText(text) => rich_text_to_gpui(text).into_any_element(),
        Node::TextInput(input) => text_input_to_gpui(input, viewport_columns),
        Node::Container(container) => {
            let mut out = div();
            if let Some(bg) = container.style.bg {
                out = out.bg(gpui::rgb(bg.0));
            }
            if let Some(text_color) = container.style.text_color {
                out = out.text_color(gpui::rgb(text_color.0));
            }
            out.child(node_to_gpui(*container.child, viewport_columns))
                .into_any_element()
        }
        Node::ScrollView(scroll) => {
            const LINE_HEIGHT_PX: f32 = 18.0;

            let mut out = div().overflow_hidden();
            out = out.w_full().flex_none();
            if let Some(lines) = scroll.viewport_lines {
                out = out.h(gpui::px(lines as f32 * LINE_HEIGHT_PX));
            }

            let mut inner = div()
                .relative()
                .w_full()
                .child(node_to_gpui(*scroll.child, viewport_columns));
            if scroll.offset_lines > 0 {
                inner = inner.top(gpui::px(-(scroll.offset_lines as f32 * LINE_HEIGHT_PX)));
            }

            out.child(inner).into_any_element()
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
                out = out.child(node_to_gpui(child, viewport_columns));
            }
            out.into_any_element()
        }
    }
}

#[cfg(feature = "backend-gpui")]
fn text_input_to_gpui(input: crate::TextInput, viewport_columns: usize) -> gpui::AnyElement {
    use gpui::{IntoElement, ParentElement, Styled, div};

    let border = gpui::rgb(0x30363d);
    div()
        .flex()
        .border_1()
        .border_color(border)
        .font_family("DejaVu Sans Mono")
        .child(
            div()
                .flex_none()
                .px_2()
                .border_r_1()
                .border_color(border)
                .text_color(gpui::rgb(0x6e7681))
                .child(rich_text_to_gpui(
                    input.to_wrapped_gutter_with_pipe_rich_text(viewport_columns),
                )),
        )
        .child(div().flex_1().px_2().child(rich_text_to_gpui(
            input.to_wrapped_content_rich_text(viewport_columns),
        )))
        .into_any_element()
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
