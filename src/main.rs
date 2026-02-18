use clap::Parser;
use xpui::IntoNode;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long, help = "Run with graphics backend (gpui)")]
    graphics: bool,
}

struct DemoApp {
    list: xpui::FocusListState,
    focus: xpui::FocusState,
    input: xpui::TextInputState,
}

impl DemoApp {
    const INPUT_CONTAINER_ID: u64 = 10;
    const INPUT_ID: u64 = 1;
    const SCROLL_ID: u64 = 2;
    const ITEM_GAP_LINES: u16 = 1;
    const FIRST_ITEM_ID: u64 = 1000;

    fn new() -> Self {
        let mut heights = Vec::new();
        for i in 0..24 {
            heights.push(Self::item_line_height(i));
        }

        let list = xpui::FocusListState::new(heights, 8, Self::ITEM_GAP_LINES);
        let mut focus = xpui::FocusState::default();
        focus.set_focused(xpui::FocusId(Self::INPUT_ID));

        Self {
            list,
            focus,
            input: xpui::TextInputState::default(),
        }
    }

    fn item_focus_id(index: u16) -> xpui::FocusId {
        xpui::FocusId(Self::FIRST_ITEM_ID + index as u64)
    }

    fn focused_index(&self) -> Option<u16> {
        let id = self.focus.focused()?.0;
        let end = Self::FIRST_ITEM_ID + self.list.item_count() as u64;
        if (Self::FIRST_ITEM_ID..end).contains(&id) {
            Some((id - Self::FIRST_ITEM_ID) as u16)
        } else {
            None
        }
    }

    fn is_input_focused(&self) -> bool {
        self.focus.focused() == Some(xpui::FocusId(Self::INPUT_ID))
    }

    fn is_input_container_focused(&self) -> bool {
        self.focus.focused() == Some(xpui::FocusId(Self::INPUT_CONTAINER_ID))
    }

    fn is_scroll_focused(&self) -> bool {
        self.focus.focused() == Some(xpui::FocusId(Self::SCROLL_ID))
    }

    fn item_line_height(index: u16) -> u16 {
        if index % 7 == 0 {
            3
        } else if index % 3 == 0 {
            2
        } else {
            1
        }
    }
}

impl xpui::UiApp for DemoApp {
    fn render(&mut self) -> xpui::Node {
        if let Some(index) = self.focused_index() {
            self.list.set_focused_index(index);
        }

        let max_offset = self.list.max_scroll_offset();
        let focused = self.focused_index();
        let input_focused = self.is_input_focused();
        let input_container_focused = self.is_input_container_focused();
        let scroll_focused = self.is_scroll_focused();

        let mut list = xpui::column().gap(Self::ITEM_GAP_LINES as u8);
        for i in 0..self.list.item_count() {
            let is_focused = focused == Some(i);
            let body = match self.list.item_height(i) {
                1 => format!("{} Item #{:02}", if is_focused { "▶" } else { " " }, i + 1),
                2 => format!(
                    "{} Item #{:02}\n   details: two-line row",
                    if is_focused { "▶" } else { " " },
                    i + 1
                ),
                _ => format!(
                    "{} Item #{:02}\n   details: three-line row\n   meta: multiline focus test",
                    if is_focused { "▶" } else { " " },
                    i + 1
                ),
            };
            list = list.child(
                xpui::container(xpui::text(body))
                    .focus(Self::item_focus_id(i))
                    .style(if is_focused {
                        xpui::BoxStyle::default()
                            .bg(xpui::rgb(0x1f2a36))
                            .text_color(xpui::rgb(0xb3e3ff))
                    } else {
                        xpui::BoxStyle::default()
                    }),
            );
        }

        xpui::container(
            xpui::column()
                .gap(2)
                .child(
                    xpui::text("Focusable text input + scrollview")
                        .run(" (terminal-first)", xpui::TextStyle::new().bold()),
                )
                .child(xpui::text(
                    "Tab/Shift+Tab focus, arrows/PgUp/PgDn/Home/End on list, Esc to quit.",
                ))
                .child(xpui::text(format!(
                    "Input value: {:?} (cursor={})",
                    self.input.value(),
                    self.input.cursor()
                )))
                .child(
                    xpui::container(
                        xpui::text_input_from_state(&self.input)
                            .placeholder("여기에 입력...")
                            .focus(xpui::FocusId(Self::INPUT_ID))
                            .focused(input_focused),
                    )
                    .focus(xpui::FocusId(Self::INPUT_CONTAINER_ID))
                    .style(if input_focused || input_container_focused {
                        xpui::BoxStyle::default()
                            .bg(xpui::rgb(0x1f2a36))
                            .text_color(xpui::rgb(0xb3e3ff))
                    } else {
                        xpui::BoxStyle::default()
                    }),
                )
                .child(xpui::text(format!(
                    "Scroll offset={}/{} (viewport={})",
                    self.list.scroll_offset(),
                    max_offset,
                    self.list.viewport_lines()
                )))
                .child(
                    xpui::container(
                        xpui::scroll_view(list)
                            .focus(xpui::FocusId(Self::SCROLL_ID))
                            .viewport_lines(self.list.viewport_lines())
                            .offset_lines(self.list.scroll_offset()),
                    )
                    .style(if scroll_focused {
                        xpui::BoxStyle::default()
                            .bg(xpui::rgb(0x1f2a36))
                            .text_color(xpui::rgb(0xb3e3ff))
                    } else {
                        xpui::BoxStyle::default()
                    }),
                ),
        )
        .style(
            xpui::BoxStyle::default()
                .bg(xpui::rgb(0x101418))
                .text_color(xpui::rgb(0xe6edf3)),
        )
        .into_node()
    }

    fn on_input(&mut self, event: xpui::UiInputEvent) {
        if self.is_input_focused() && self.input.handle_input(event) {
            return;
        }

        if let Some(index) = self.focused_index() {
            self.list.set_focused_index(index);

            match event {
                xpui::UiInputEvent::Key(xpui::UiKeyInput::Up) => self.list.move_focus_by(-1),
                xpui::UiInputEvent::Key(xpui::UiKeyInput::Down) => self.list.move_focus_by(1),
                xpui::UiInputEvent::Key(xpui::UiKeyInput::PageUp) => self
                    .list
                    .move_focus_by(-(self.list.viewport_lines() as i16)),
                xpui::UiInputEvent::Key(xpui::UiKeyInput::PageDown) => {
                    self.list.move_focus_by(self.list.viewport_lines() as i16)
                }
                xpui::UiInputEvent::Key(xpui::UiKeyInput::Home) => self.list.set_focused_index(0),
                xpui::UiInputEvent::Key(xpui::UiKeyInput::End) => self
                    .list
                    .set_focused_index(self.list.item_count().saturating_sub(1)),
                xpui::UiInputEvent::Key(xpui::UiKeyInput::Tab | xpui::UiKeyInput::BackTab) => {
                    self.list.ensure_focused_visible()
                }
                xpui::UiInputEvent::ScrollLines(lines) if lines < 0 => {
                    self.list.move_focus_by(-(lines.unsigned_abs() as i16))
                }
                xpui::UiInputEvent::ScrollLines(lines) if lines > 0 => {
                    self.list.move_focus_by(lines)
                }
                _ => {}
            }

            self.focus
                .set_focused(Self::item_focus_id(self.list.focused_index()));
        }
    }

    fn focus_state(&mut self) -> Option<&mut xpui::FocusState> {
        Some(&mut self.focus)
    }
}

fn main() {
    let args = Args::parse();

    if args.graphics {
        xpui::run_gpui(DemoApp::new());
    } else {
        xpui::run_cpui(DemoApp::new());
    }
}
