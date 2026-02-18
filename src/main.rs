use clap::Parser;
use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;
use xpui::IntoNode;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long, help = "Run with graphics backend (gpui)")]
    graphics: bool,
}

struct DemoApp {
    window_size: xpui::WindowSize,
    list_binding: xpui::FocusListBinding,
    list: xpui::FocusListState,
    focus: xpui::FocusState,
    input: xpui::TextInputState,
    messages: Vec<String>,
    selected_model: String,
}

impl DemoApp {
    const INPUT_CONTAINER_ID: u64 = 10;
    const INPUT_ID: u64 = 1;
    const SCROLL_ID: u64 = 2;
    const ITEM_GAP_LINES: u16 = 1;
    const FIRST_ITEM_ID: u64 = 1000;

    fn new() -> Self {
        let list_binding = xpui::FocusListBinding::new(Self::FIRST_ITEM_ID);
        let messages = vec![
            "assistant: 안녕하세요! 무엇을 도와드릴까요?".to_string(),
            "user: 포커스 트리 네비게이션을 개선하고 싶어요.".to_string(),
            "assistant: 좋아요. Enter로 하위 진입, Esc로 상위 복귀 모델로 가죠.".to_string(),
        ];
        let heights = messages
            .iter()
            .map(|message| Self::wrapped_line_count(&Self::format_history_row(message, false), 78))
            .collect::<Vec<_>>();
        let list = xpui::FocusListState::new(heights, 8, Self::ITEM_GAP_LINES);
        let mut focus = xpui::FocusState::default();
        focus.set_focused(xpui::FocusId(Self::INPUT_ID));

        Self {
            window_size: xpui::WindowSize::default(),
            list_binding,
            list,
            focus,
            input: xpui::TextInputState::default(),
            messages,
            selected_model: "gpt-4.1".to_string(),
        }
    }

    fn is_input_focused(&self) -> bool {
        self.focus.is_focused(xpui::FocusId(Self::INPUT_ID))
    }

    fn is_input_container_focused(&self) -> bool {
        self.focus
            .is_focused(xpui::FocusId(Self::INPUT_CONTAINER_ID))
    }

    fn is_scroll_focused(&self) -> bool {
        self.focus.is_focused(xpui::FocusId(Self::SCROLL_ID))
    }

    fn input_visual_metrics(&self, total_width: usize) -> (u16, u16) {
        let lines: Vec<&str> = self.input.value().split('\n').collect();
        let line_count = lines.len().max(1);
        let gutter_digits = line_count.to_string().len();
        let content_width = total_width.saturating_sub(gutter_digits + 3).max(1);

        let mut total_visual = 0u16;
        let mut cursor_visual = 0u16;
        let mut cursor_left = self.input.cursor();

        for line in lines {
            let mut wraps = 1u16;
            let mut col = 0usize;
            let mut line_chars = 0usize;
            for ch in line.chars() {
                let w = UnicodeWidthChar::width(ch).unwrap_or(0);
                if col > 0 && col.saturating_add(w) > content_width {
                    wraps = wraps.saturating_add(1);
                    col = 0;
                }
                col = col.saturating_add(w);
                line_chars += 1;
            }

            if cursor_left <= line_chars {
                let mut ccol = 0usize;
                let mut cwrap = 0u16;
                for ch in line.chars().take(cursor_left) {
                    let w = UnicodeWidthChar::width(ch).unwrap_or(0);
                    if ccol > 0 && ccol.saturating_add(w) > content_width {
                        cwrap = cwrap.saturating_add(1);
                        ccol = 0;
                    }
                    ccol = ccol.saturating_add(w);
                }
                cursor_visual = total_visual.saturating_add(cwrap);
                return (total_visual.saturating_add(wraps), cursor_visual);
            }

            cursor_left = cursor_left.saturating_sub(line_chars.saturating_add(1));
            total_visual = total_visual.saturating_add(wraps);
        }

        (total_visual.max(1), cursor_visual)
    }

    fn format_history_row(message: &str, focused: bool) -> String {
        let mut lines = message.lines();
        let first = lines.next().unwrap_or_default();
        let mut out = format!("{} {}", if focused { "▶" } else { " " }, first);
        for line in lines {
            out.push('\n');
            out.push_str("  ");
            out.push_str(line);
        }
        out
    }

    fn wrapped_line_count(text: &str, wrap_width: usize) -> u16 {
        if wrap_width == 0 {
            return 1;
        }
        let mut lines = 1u16;
        let mut col = 0usize;
        for ch in text.chars() {
            if ch == '\n' {
                lines = lines.saturating_add(1);
                col = 0;
                continue;
            }
            let w = UnicodeWidthChar::width(ch).unwrap_or(0);
            if col > 0 && col.saturating_add(w) > wrap_width {
                lines = lines.saturating_add(1);
                col = 0;
            }
            col = col.saturating_add(w);
        }
        lines.max(1)
    }

    fn recalc_history_heights(&mut self) {
        let wrap_width = (self.window_size.width as usize).saturating_sub(2).max(1);
        let heights = self
            .messages
            .iter()
            .map(|message| {
                Self::wrapped_line_count(&Self::format_history_row(message, false), wrap_width)
            })
            .collect::<Vec<_>>();
        self.list.set_item_heights(heights);
    }

    fn submit_message(&mut self) {
        let text = self.input.value().trim();
        if text.is_empty() {
            return;
        }
        self.messages.push(format!("you: {}", text));
        self.recalc_history_heights();
        self.input.set_value("");
    }

    fn bottom_bar_text(
        &self,
        width: usize,
        input_focused: bool,
        input_container_focused: bool,
        scroll_focused: bool,
    ) -> String {
        let usage_top = if input_focused {
            "Alt+Enter send • Enter newline • Esc exit input"
        } else if input_container_focused {
            "Enter edit input • Down move to history • Esc step out"
        } else if scroll_focused {
            "Enter focus item • Up/Down/Page scroll • Esc step out"
        } else {
            "Up/Down move • Enter enter child • Esc step out"
        };
        let usage_mid = "Tab/Shift+Tab focus • Esc x2 on root quits";
        let usage_bot = if input_focused {
            "Focus: Input"
        } else if input_container_focused {
            "Focus: Input container"
        } else if scroll_focused {
            "Focus: History scroll"
        } else {
            "Focus: History item"
        };
        let model = format!("Model: {}", self.selected_model);
        let line1 = Self::left_right_line(usage_top, &model, width);

        format!("{line1}\n{usage_mid}\n{usage_bot}")
    }

    fn left_right_line(left: &str, right: &str, width: usize) -> String {
        let left_w = left.width();
        let right_w = right.width();
        if left_w + right_w + 1 > width {
            return format!("{left} {right}");
        }
        let spaces = width - left_w - right_w;
        format!("{left}{}{right}", " ".repeat(spaces))
    }
}

impl xpui::UiApp for DemoApp {
    fn set_window_size(&mut self, size: xpui::WindowSize) {
        self.window_size = size;
    }

    fn render(&mut self) -> xpui::Node {
        self.recalc_history_heights();
        self.list_binding
            .sync_list_from_focus(&self.focus, &mut self.list);

        let focused = self
            .list_binding
            .focused_index(&self.focus, self.list.item_count());
        let input_focused = self.is_input_focused();
        let input_container_focused = self.is_input_container_focused();
        let scroll_focused = self.is_scroll_focused();
        let dynamic_input_max = ((self.window_size.height * 0.20).floor() as u16).max(5);
        let input_wrap_width = (self.window_size.width as usize).max(8);
        let (input_visual_lines, cursor_line) = self.input_visual_metrics(input_wrap_width);
        let input_viewport_lines = input_visual_lines.clamp(1, dynamic_input_max);
        let input_offset_lines = cursor_line
            .saturating_add(1)
            .saturating_sub(input_viewport_lines);
        let terminal_lines = (self.window_size.height as u16).max(1);
        let reserved_without_history = 15u16.saturating_add(input_viewport_lines);
        let history_viewport_lines = terminal_lines
            .saturating_sub(reserved_without_history)
            .max(3);
        self.list.set_viewport_lines(history_viewport_lines);

        let mut list = xpui::column().gap(Self::ITEM_GAP_LINES as u8);
        for (i, message) in self.messages.iter().enumerate() {
            let i = i as u16;
            let is_focused = focused == Some(i);
            let body = Self::format_history_row(message, is_focused);
            list = list.child(
                xpui::container(xpui::text(body))
                    .focus(self.list_binding.focus_id(i))
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
                .gap(1)
                .child(
                    xpui::text("Focusable text input + scrollview")
                        .run(" (terminal-first)", xpui::TextStyle::new().bold()),
                )
                .child(xpui::text(
                    "Tab/Shift+Tab focus, arrows/PgUp/PgDn/Home/End on list, Esc to quit.",
                ))
                .child(xpui::text("Chat history"))
                .child(
                    xpui::container(
                        xpui::scroll_view(list)
                            .focus(xpui::FocusId(Self::SCROLL_ID))
                            .viewport_lines(history_viewport_lines)
                            .offset_lines(self.list.scroll_offset()),
                    )
                    .style(if scroll_focused {
                        xpui::BoxStyle::default()
                            .bg(xpui::rgb(0x1f2a36))
                            .text_color(xpui::rgb(0xb3e3ff))
                    } else {
                        xpui::BoxStyle::default()
                    }),
                )
                .child(
                    xpui::container(
                        xpui::scroll_view(
                            xpui::text_input_from_state(&self.input)
                                .placeholder("여기에 입력...")
                                .focus(xpui::FocusId(Self::INPUT_ID))
                                .focused(input_focused)
                                .visible_offset_lines(input_offset_lines),
                        )
                        .viewport_lines(input_viewport_lines)
                        .offset_lines(input_offset_lines),
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
                .child(
                    xpui::container(
                        xpui::scroll_view(xpui::text(self.bottom_bar_text(
                            self.window_size.width as usize,
                            input_focused,
                            input_container_focused,
                            scroll_focused,
                        )))
                        .viewport_lines(3),
                    )
                    .style(
                        xpui::BoxStyle::default()
                            .bg(xpui::rgb(0x161b22))
                            .text_color(xpui::rgb(0xc9d1d9)),
                    ),
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
        let line_count = self.input.value().split('\n').count().max(1);
        let gutter_digits = line_count.to_string().len();
        let input_total_width = (self.window_size.width as usize).max(8);
        let input_content_width = input_total_width.saturating_sub(gutter_digits + 3).max(1);
        self.input.set_soft_wrap_width(Some(input_content_width));

        if self.is_input_focused()
            && matches!(event, xpui::UiInputEvent::Key(xpui::UiKeyInput::Submit))
        {
            self.submit_message();
            return;
        }
        if self.is_input_focused() && self.input.handle_input(event) {
            return;
        }
        let _ = self
            .list_binding
            .handle_input(&mut self.focus, &mut self.list, event);
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
        println!("     ..::.");
        println!("   .-=+++=-:     Hello");
        println!("  .-+**#**+-.    loopcode session ended");
        println!("  .-+*###*+-.    run again: cargo run");
        println!("   :-=+++=-:");
        println!("     .:::. ");
    }
}
