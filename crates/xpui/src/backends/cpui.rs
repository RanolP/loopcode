use cpui::{AppContext, IntoElement};

use crate::{
    backend::Backend,
    node::{Axis, Node, RichText, TextInput},
    runtime::{FocusEntry, FocusKind, UiApp, UiInputEvent, UiKeyInput, WindowSize},
    style::{Rgb, TextStyle},
};

pub struct CpuiBackend;

impl Backend for CpuiBackend {
    type Output = cpui::AnyElement;

    fn render_node(&mut self, node: Node) -> Self::Output {
        node_to_cpui(node, 80)
    }
}

pub(crate) fn run_cpui<A: UiApp + 'static>(app: A, size: WindowSize) {
    struct HostEntity<A: UiApp + 'static>(cpui::Entity<Host<A>>);

    impl<A: UiApp + 'static> Clone for HostEntity<A> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    struct Host<A> {
        app: A,
        focus_order: Vec<FocusEntry>,
        window_size: WindowSize,
        esc_armed: bool,
    }

    impl<A: UiApp + 'static> cpui::Render for Host<A> {
        fn render(
            &mut self,
            _window: &mut cpui::Window,
            _cx: &mut cpui::Context<'_, Self>,
        ) -> impl cpui::IntoElement {
            self.app.set_window_size(self.window_size);
            let node = self.app.render();

            let mut entries = Vec::new();
            node.collect_focus_entries(&mut entries);
            self.focus_order = entries.clone();

            if let Some(focus) = self.app.focus_state() {
                focus.ensure_valid(&entries);
            }

            node_to_cpui(node, self.window_size.width.max(1.0) as usize)
        }
    }

    cpui::Application::new().run_with_input_handler(
        move |cx: &mut cpui::App| {
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
                |_window, cx| {
                    let entity = cx.new(|_cx| Host {
                        app,
                        focus_order: Vec::new(),
                        window_size: size,
                        esc_armed: false,
                    });
                    cx.set_global(HostEntity(entity.clone()));
                    entity
                },
            );

            cx.activate(true);
        },
        move |cx: &mut cpui::App, event| {
            let Some(host_entity) = cx.global::<HostEntity<A>>().cloned().map(|h| h.0) else {
                return false;
            };

            let mut should_quit = false;
            let _ = cx.update_entity(&host_entity, |host, _| {
                let handled_focus = match event {
                    cpui::InputEvent::Key(cpui::KeyInput::Tab) => {
                        if let Some(focus) = host.app.focus_state() {
                            focus.focus_next(&host.focus_order);
                            true
                        } else {
                            false
                        }
                    }
                    cpui::InputEvent::Key(cpui::KeyInput::BackTab) => {
                        if let Some(focus) = host.app.focus_state() {
                            focus.focus_prev(&host.focus_order);
                            true
                        } else {
                            false
                        }
                    }
                    cpui::InputEvent::Key(cpui::KeyInput::Left | cpui::KeyInput::Up) => {
                        if let Some(focus) = host.app.focus_state() {
                            let is_text_input = focus
                                .focused_entry(&host.focus_order)
                                .is_some_and(|entry| entry.kind == FocusKind::TextInput);
                            if !is_text_input {
                                focus.focus_prev_sibling(&host.focus_order)
                                    || focus.focus_prev_peer_branch(&host.focus_order)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                    cpui::InputEvent::Key(cpui::KeyInput::Right | cpui::KeyInput::Down) => {
                        if let Some(focus) = host.app.focus_state() {
                            let is_text_input = focus
                                .focused_entry(&host.focus_order)
                                .is_some_and(|entry| entry.kind == FocusKind::TextInput);
                            if !is_text_input {
                                focus.focus_next_sibling(&host.focus_order)
                                    || focus.focus_next_peer_branch(&host.focus_order)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                    cpui::InputEvent::Key(cpui::KeyInput::Enter) => {
                        if let Some(focus) = host.app.focus_state() {
                            let is_text_input = focus
                                .focused_entry(&host.focus_order)
                                .is_some_and(|entry| entry.kind == FocusKind::TextInput);
                            if !is_text_input {
                                focus.focus_first_child(&host.focus_order)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                    cpui::InputEvent::Key(cpui::KeyInput::Esc) => {
                        if let Some(focus) = host.app.focus_state() {
                            if focus.focus_parent(&host.focus_order) {
                                host.esc_armed = false;
                            } else if host.esc_armed {
                                should_quit = true;
                            } else {
                                host.esc_armed = true;
                            }
                            true
                        } else if host.esc_armed {
                            should_quit = true;
                            true
                        } else {
                            host.esc_armed = true;
                            true
                        }
                    }
                    _ => false,
                };

                if !handled_focus {
                    host.esc_armed = false;
                    if let Some(event) = from_cpui_input(event) {
                        host.app.on_input(event);
                    }
                }
            });

            should_quit
        },
    );
}

fn node_to_cpui(node: Node, viewport_columns: usize) -> cpui::AnyElement {
    match node {
        Node::Empty => cpui::AnyElement::Empty,
        Node::RichText(text) => text_to_cpui(text).into_any_element(),
        Node::TextInput(input) => text_input_to_cpui(input, viewport_columns),
        Node::Container(container) => {
            let mut out = cpui::div();
            if let Some(bg) = container.style.bg {
                out = out.bg(to_cpui_color(bg));
            }
            if let Some(text_color) = container.style.text_color {
                out = out.text_color(to_cpui_color(text_color));
            }
            out.child(node_to_cpui(*container.child, viewport_columns))
                .into_any_element()
        }
        Node::ScrollView(scroll) => {
            let mut out = cpui::scroll_view(node_to_cpui(*scroll.child, viewport_columns))
                .offset_lines(scroll.offset_lines);
            if let Some(lines) = scroll.viewport_lines {
                out = out.viewport_lines(lines);
            }
            out.into_any_element()
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
                out = out.child(node_to_cpui(child, viewport_columns));
            }

            out.into_any_element()
        }
    }
}

fn text_input_to_cpui(input: TextInput, viewport_columns: usize) -> cpui::AnyElement {
    text_to_cpui(input.to_wrapped_rich_text(viewport_columns)).into_any_element()
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

fn from_cpui_input(event: cpui::InputEvent) -> Option<UiInputEvent> {
    match event {
        cpui::InputEvent::Key(key) => {
            let mapped = match key {
                cpui::KeyInput::Tab => UiKeyInput::Tab,
                cpui::KeyInput::BackTab => UiKeyInput::BackTab,
                cpui::KeyInput::Left => UiKeyInput::Left,
                cpui::KeyInput::Right => UiKeyInput::Right,
                cpui::KeyInput::WordLeft => UiKeyInput::WordLeft,
                cpui::KeyInput::WordRight => UiKeyInput::WordRight,
                cpui::KeyInput::Up => UiKeyInput::Up,
                cpui::KeyInput::Down => UiKeyInput::Down,
                cpui::KeyInput::PageUp => UiKeyInput::PageUp,
                cpui::KeyInput::PageDown => UiKeyInput::PageDown,
                cpui::KeyInput::Home => UiKeyInput::Home,
                cpui::KeyInput::End => UiKeyInput::End,
                cpui::KeyInput::Backspace => UiKeyInput::Backspace,
                cpui::KeyInput::BackspaceWord => UiKeyInput::BackspaceWord,
                cpui::KeyInput::Delete => UiKeyInput::Delete,
                cpui::KeyInput::Enter => UiKeyInput::Enter,
                cpui::KeyInput::Esc => UiKeyInput::Esc,
                cpui::KeyInput::Char(ch) => UiKeyInput::Char(ch),
            };
            Some(UiInputEvent::Key(mapped))
        }
        cpui::InputEvent::ScrollLines(lines) => Some(UiInputEvent::ScrollLines(lines)),
    }
}
