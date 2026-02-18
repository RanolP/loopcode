use cpui::{AppContext, IntoElement};

use crate::{
    backend::Backend,
    node::{Axis, Icon, IconName, Node, RichText, TextInput},
    runtime::{FocusEntry, FocusNavOutcome, UiApp, UiInputEvent, UiKeyInput, WindowSize},
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
    }

    impl<A: UiApp + 'static> cpui::Render for Host<A> {
        fn render(
            &mut self,
            window: &mut cpui::Window,
            _cx: &mut cpui::Context<'_, Self>,
        ) -> impl cpui::IntoElement {
            if let Ok((w, h)) = window.terminal_size() {
                self.window_size = WindowSize {
                    width: w as f32,
                    height: h as f32,
                };
            }
            self.app.set_window_size(self.window_size);
            let node = self.app.render();

            let mut entries = Vec::new();
            node.collect_focus_entries(&mut entries);
            self.focus_order = entries.clone();

            if let Some(focus) = self.app.focus_state() {
                focus.ensure_valid(&entries);
            }
            self.app.on_focus_entries(&entries);

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
                    let entity = cx.create_entity(|_cx| Host {
                        app,
                        focus_order: Vec::new(),
                        window_size: size,
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
            cx.update_entity(&host_entity, |host, _| {
                let Some(event) = from_cpui_input(event) else {
                    return;
                };

                let nav_outcome = if let Some(focus) = host.app.focus_state() {
                    focus.handle_navigation(event, &host.focus_order)
                } else {
                    FocusNavOutcome::Ignored
                };

                match nav_outcome {
                    FocusNavOutcome::Ignored => host.app.on_input(event),
                    FocusNavOutcome::Handled => {}
                    FocusNavOutcome::RequestQuit => should_quit = true,
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
        Node::Icon(icon) => icon_to_cpui(icon).into_any_element(),
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

fn icon_to_cpui(icon: Icon) -> cpui::StyledText {
    let glyph = match icon.name {
        IconName::Search => "󰍉",
        IconName::Send => "󰒊",
        IconName::Robot => "󰚩",
        IconName::Info => "󰋼",
        IconName::Warning => "󰀪",
        IconName::Error => "󰅚",
        IconName::Check => "󰄬",
        IconName::ChevronRight => "󰅂",
        IconName::ChevronDown => "󰅀",
    };
    let mut style = cpui::TextStyle::new();
    if let Some(color) = icon.color {
        style = style.color(to_cpui_color(color));
    }
    cpui::StyledText::empty().push_run(glyph, style)
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
    if style.cursor_anchor {
        out = out.cursor_anchor(style.cursor_after);
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
                cpui::KeyInput::ShiftTab => UiKeyInput::ShiftTab,
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
                cpui::KeyInput::Submit => UiKeyInput::Submit,
                cpui::KeyInput::Esc => UiKeyInput::Esc,
                cpui::KeyInput::Interrupt => UiKeyInput::Interrupt,
                cpui::KeyInput::Char(ch) => UiKeyInput::Char(ch),
            };
            Some(UiInputEvent::Key(mapped))
        }
        cpui::InputEvent::ScrollLines(lines) => Some(UiInputEvent::ScrollLines(lines)),
        cpui::InputEvent::MouseDown { x, y } => Some(UiInputEvent::MouseDown { x, y }),
        cpui::InputEvent::Tick => Some(UiInputEvent::Tick),
    }
}
