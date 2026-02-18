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

#[derive(Clone, Copy)]
enum AgentMode {
    Safe,
    Autonomous,
    Jailbreaking,
}

impl AgentMode {
    fn cycle(self) -> Self {
        match self {
            Self::Safe => Self::Autonomous,
            Self::Autonomous => Self::Jailbreaking,
            Self::Jailbreaking => Self::Safe,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Safe => "Safest",
            Self::Autonomous => "Autonomous",
            Self::Jailbreaking => "Jailbreak",
        }
    }
}

struct ChatState {
    input: xpui::TextInputState,
    history: ChatHistory,
    selected_model: xpui::signal::Signal<String>,
    history_heights_memo: xpui::signal::Memo<(u64, usize), Vec<u16>>,
}

impl ChatState {
    fn new(events: xpui::signal::EventSignal<HistoryEvent>) -> Self {
        let history = ChatHistory::new(vec![
            "assistant: 안녕하세요! 무엇을 도와드릴까요?".to_string(),
            "user: 포커스 트리 네비게이션을 개선하고 싶어요.".to_string(),
            "assistant: 좋아요. Enter로 하위 진입, Esc로 상위 복귀 모델로 가죠.".to_string(),
        ], events);
        history.reset_to_index(history.len().saturating_sub(1));

        Self {
            input: xpui::TextInputState::default(),
            history,
            selected_model: xpui::signal::Signal::from("OpenRouter GPT-4.1".to_string()),
            history_heights_memo: xpui::signal::Memo::new(),
        }
    }

    fn submit_input(&mut self) -> bool {
        let text = self.input.value().trim();
        if text.is_empty() {
            return false;
        }
        self.history.append_user(format!("you: {}", text));
        self.input.set_value("");
        true
    }
}

#[derive(Clone, Copy)]
enum HistoryEvent {
    UserAppended,
    Reset,
}

pub(crate) struct ChatHistory {
    messages: xpui::signal::VecSignal<String>,
    events: xpui::signal::EventSignal<HistoryEvent>,
}

impl ChatHistory {
    pub(crate) fn new(initial: Vec<String>, events: xpui::signal::EventSignal<HistoryEvent>) -> Self {
        Self {
            messages: xpui::signal::VecSignal::from(initial),
            events,
        }
    }

    pub(crate) fn append_user(&self, message: String) {
        self.messages.push(message);
        self.events.emit(HistoryEvent::UserAppended);
    }

    pub(crate) fn reset_to_index(&self, index: usize) {
        self.messages.update(|items| {
            if let Some(keep) = index.checked_add(1)
                && keep < items.len()
            {
                items.truncate(keep);
            }
        });
        self.events.emit(HistoryEvent::Reset);
    }

    pub(crate) fn len(&self) -> usize {
        self.messages.len()
    }

    pub(crate) fn version(&self) -> u64 {
        self.messages.version()
    }

    pub(crate) fn borrow(&self) -> std::cell::Ref<'_, Vec<String>> {
        self.messages.borrow()
    }
}

struct FocusUiState {
    list_binding: xpui::FocusListBinding,
    list: xpui::FocusListState,
    focus: xpui::FocusState,
}

impl FocusUiState {
    fn new(initial_heights: Vec<u16>, viewport: u16, gap: u16) -> Self {
        let list_binding = xpui::FocusListBinding::new(DemoApp::FIRST_ITEM_ID);
        let list = xpui::FocusListState::new(initial_heights, viewport, gap);
        let mut focus = xpui::FocusState::default();
        focus.set_focused(xpui::FocusId(DemoApp::INPUT_ID));
        Self {
            list_binding,
            list,
            focus,
        }
    }
}

struct DemoApp {
    window_size: xpui::WindowSize,
    chat: ChatState,
    history_events: xpui::signal::EventSignal<HistoryEvent>,
    nav: FocusUiState,
    is_vscode_terminal: bool,
    current_dir: String,
    mode: AgentMode,
    input_scroll_offset: u16,
}

impl DemoApp {
    const INPUT_CONTAINER_ID: u64 = 10;
    const INPUT_ID: u64 = 1;
    const SCROLL_ID: u64 = 2;
    const ITEM_GAP_LINES: u16 = 1;
    const FIRST_ITEM_ID: u64 = 1000;

    fn new() -> Self {
        let history_events = xpui::signal::EventSignal::new();
        let chat = ChatState::new(history_events.clone());
        let heights = chat
            .history
            .borrow()
            .iter()
            .map(|message| Self::wrapped_line_count(&Self::format_history_row(message, false), 78))
            .collect::<Vec<_>>();
        let nav = FocusUiState::new(heights, 8, Self::ITEM_GAP_LINES);

        Self {
            window_size: xpui::WindowSize::default(),
            chat,
            history_events,
            nav,
            is_vscode_terminal: std::env::var("TERM_PROGRAM")
                .map(|v| v.eq_ignore_ascii_case("vscode"))
                .unwrap_or(false),
            current_dir: std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| ".".to_string()),
            mode: AgentMode::Safe,
            input_scroll_offset: 0,
        }
    }

    fn is_input_focused(&self) -> bool {
        self.nav.focus.is_focused(xpui::FocusId(Self::INPUT_ID))
    }

    fn is_input_container_focused(&self) -> bool {
        self.nav
            .focus
            .is_focused(xpui::FocusId(Self::INPUT_CONTAINER_ID))
    }

    fn is_scroll_focused(&self) -> bool {
        self.nav.focus.is_focused(xpui::FocusId(Self::SCROLL_ID))
    }

    fn input_visual_metrics(&self, total_width: usize) -> (u16, u16) {
        let lines: Vec<&str> = self.chat.input.value().split('\n').collect();
        let line_count = lines.len().max(1);
        let gutter_digits = line_count.to_string().len();
        let content_width = total_width.saturating_sub(gutter_digits + 3).max(1);

        let mut total_visual = 0u16;
        let mut cursor_visual = 0u16;
        let mut cursor_left = self.chat.input.cursor();
        let mut cursor_found = false;

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

            if !cursor_found && cursor_left <= line_chars {
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
                cursor_found = true;
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

    fn usage_top_parts(
        &self,
        input_focused: bool,
        input_container_focused: bool,
        scroll_focused: bool,
    ) -> Vec<(&'static str, &'static str)> {
        if input_focused {
            if self.is_vscode_terminal {
                vec![
                    ("Alt+Enter", "send"),
                    ("Enter", "newline"),
                    ("Esc", "exit input"),
                ]
            } else {
                vec![
                    ("Ctrl+Enter", "send"),
                    ("Enter", "newline"),
                    ("Esc", "exit input"),
                ]
            }
        } else if input_container_focused {
            vec![
                ("Enter", "focus input"),
                ("Up", "see history"),
            ]
        } else if scroll_focused {
            vec![
                ("Enter", "focus and scroll"),
                ("Down", "return to input"),
            ]
        } else {
            vec![
                ("Up/Down", "navigate"),
                ("Esc", "return to chat list"),
            ]
        }
    }

    fn bottom_bar_node(
        &self,
        width: usize,
        input_focused: bool,
        input_container_focused: bool,
        scroll_focused: bool,
    ) -> xpui::Node {
        let parts = self.usage_top_parts(input_focused, input_container_focused, scroll_focused);
        let (provider, model_name) = self.selected_model_parts();
        let usage_top_plain = parts
            .iter()
            .map(|(k, a)| format!("{k} {a}"))
            .collect::<Vec<_>>()
            .join(" • ");
        let usage_mid_left = if self.nav.focus.quit_armed() {
            "Press Ctrl+C again to quit"
        } else {
            ""
        };
        let usage_mid_right = "45% used · $0.21";
        let model_plain = if model_name.is_empty() {
            provider.clone()
        } else {
            format!("{provider} {model_name}")
        };
        let left_w = usage_top_plain.width();
        let right_w = model_plain.width();
        let spaces = if left_w + right_w + 1 > width {
            1
        } else {
            width - left_w - right_w
        };

        let key_style = xpui::TextStyle::new().color(xpui::rgb(0xa3afbf));
        let action_style = xpui::TextStyle::new().color(xpui::rgb(0x7f8a9a));
        let dot_style = xpui::TextStyle::new().color(xpui::rgb(0x596272));
        let provider_style = xpui::TextStyle::new().color(xpui::rgb(0x8b949e));
        let name_style = xpui::TextStyle::new().color(xpui::rgb(0xc9d1d9));
        let usage_left_style = xpui::TextStyle::new().color(xpui::rgb(0x7f8a9a));
        let usage_right_style = xpui::TextStyle::new().color(xpui::rgb(0x8b949e));

        let mut line1 = xpui::text("");
        for (i, (key, action)) in parts.iter().enumerate() {
            if i > 0 {
                line1 = line1.run(" · ", dot_style.clone());
            }
            line1 = line1
                .run(*key, key_style.clone())
                .run(" ", action_style.clone())
                .run(*action, action_style.clone());
        }
        line1 = line1.run(" ".repeat(spaces), xpui::TextStyle::new());
        line1 = line1.run(provider, provider_style);
        if !model_name.is_empty() {
            line1 = line1.run(" ", xpui::TextStyle::new());
            line1 = line1.run(model_name, name_style);
        }

        let mid_left_w = usage_mid_left.width();
        let mid_right_w = usage_mid_right.width();
        let mid_spaces = if mid_left_w + mid_right_w + 1 > width {
            1
        } else {
            width - mid_left_w - mid_right_w
        };

        line1
            .run("\n", xpui::TextStyle::new())
            .run(usage_mid_left, usage_left_style)
            .run(" ".repeat(mid_spaces), xpui::TextStyle::new())
            .run(usage_mid_right, usage_right_style)
            .into_node()
    }

    fn selected_model_parts(&self) -> (String, String) {
        let raw = self.chat.selected_model.borrow().trim().to_string();
        if raw.is_empty() {
            return ("OpenRouter".to_string(), "GPT-4.1".to_string());
        }
        if let Some((provider, name)) = raw.split_once(' ') {
            (provider.to_string(), name.trim().to_string())
        } else {
            ("OpenRouter".to_string(), raw)
        }
    }

    fn mode_surface_colors(&self) -> (xpui::Rgb, xpui::Rgb) {
        match self.mode {
            AgentMode::Safe => (xpui::rgb(0x1f4d2b), xpui::rgb(0xf2fbf4)),
            AgentMode::Autonomous => (xpui::rgb(0x1f3f66), xpui::rgb(0xf2f7fc)),
            AgentMode::Jailbreaking => (xpui::rgb(0x6b2f2f), xpui::rgb(0xfcf3f3)),
        }
    }

    fn mode_tag_colors(&self) -> (xpui::Rgb, xpui::Rgb) {
        match self.mode {
            AgentMode::Safe => (xpui::rgb(0x132a13), xpui::rgb(0xb7f7c0)),
            AgentMode::Autonomous => (xpui::rgb(0x10243d), xpui::rgb(0xb3e3ff)),
            AgentMode::Jailbreaking => (xpui::rgb(0x3a1212), xpui::rgb(0xffc9c9)),
        }
    }

    fn status_bar_node(&self, width: usize) -> xpui::Node {
        let left = format!("Dir: {}", self.current_dir);
        let mode_label = self.mode.title();
        let mode_tag = format!(" {} ", "MODE");
        let mode_value = format!(" {} ", mode_label);
        let right_plain = format!("{mode_tag}{mode_value}");
        let left_w = left.width();
        let right_w = right_plain.width();
        let spaces = if left_w + right_w + 1 > width {
            1
        } else {
            width - left_w - right_w
        };
        let (tag_fg, tag_bg) = self.mode_tag_colors();
        let (value_fg, value_bg) = self.mode_surface_colors();
        let mode_tag_style = xpui::TextStyle::new().bg(tag_bg).color(tag_fg).bold();
        let mode_value_style = xpui::TextStyle::new().bg(value_bg).color(value_fg).bold();

        xpui::text(left)
            .run(" ".repeat(spaces), xpui::TextStyle::new())
            .run(mode_tag, mode_tag_style)
            .run(mode_value, mode_value_style)
            .into_node()
    }

    fn history_viewport_lines(&self) -> u16 {
        let dynamic_input_max = ((self.window_size.height * 0.20).floor() as u16).max(5);
        let input_wrap_width = (self.window_size.width as usize).max(8);
        let (input_visual_lines, _) = self.input_visual_metrics(input_wrap_width);
        let input_viewport_lines = input_visual_lines.clamp(1, dynamic_input_max);
        let terminal_lines = (self.window_size.height as u16).max(1);
        let reserved_without_history = 6u16.saturating_add(input_viewport_lines);
        terminal_lines.saturating_sub(reserved_without_history).max(3)
    }

    fn input_viewport_lines(&self) -> u16 {
        let dynamic_input_max = ((self.window_size.height * 0.20).floor() as u16).max(5);
        let input_wrap_width = (self.window_size.width as usize).max(8);
        let (input_visual_lines, _) = self.input_visual_metrics(input_wrap_width);
        input_visual_lines.clamp(1, dynamic_input_max)
    }

    fn input_layout_for_click(&self) -> (usize, u16, u16, usize) {
        let line_count = self.chat.input.value().split('\n').count().max(1);
        let gutter_digits = line_count.to_string().len();
        let input_total_width = (self.window_size.width as usize).max(8);
        let content_width = input_total_width.saturating_sub(gutter_digits + 3).max(1);
        let dynamic_input_max = ((self.window_size.height * 0.20).floor() as u16).max(5);
        let (input_visual_lines, _) = self.input_visual_metrics(input_total_width);
        let input_viewport_lines = input_visual_lines.clamp(1, dynamic_input_max);
        let input_offset_lines = self
            .input_scroll_offset
            .min(input_visual_lines.saturating_sub(input_viewport_lines));
        (content_width, input_viewport_lines, input_offset_lines, gutter_digits)
    }

    fn input_max_scroll_offset(&self) -> u16 {
        let input_total_width = (self.window_size.width as usize).max(8);
        let (input_visual_lines, _) = self.input_visual_metrics(input_total_width);
        let input_viewport_lines = self.input_viewport_lines();
        input_visual_lines.saturating_sub(input_viewport_lines)
    }

    fn clamp_input_scroll_offset(&mut self) {
        self.input_scroll_offset = self.input_scroll_offset.min(self.input_max_scroll_offset());
    }

    fn history_item_at_row(&self, row: u16) -> Option<u16> {
        let line = self.nav.list.scroll_offset().saturating_add(row);
        let mut top = 0u16;
        for index in 0..self.nav.list.item_count() {
            let height = self.nav.list.item_height(index);
            let bottom = top.saturating_add(height);
            if line >= top && line < bottom {
                return Some(index);
            }
            top = bottom.saturating_add(Self::ITEM_GAP_LINES);
        }
        None
    }

    fn is_mode_click(&self, x: u16, y: u16) -> bool {
        let width = self.window_size.width as usize;
        let height = self.window_size.height as u16;
        if width == 0 || height == 0 || y != height.saturating_sub(1) {
            return false;
        }

        let mode_label = self.mode.title();
        let mode_tag = format!(" {} ", "MODE");
        let mode_value = format!(" {} ", mode_label);
        let right_plain = format!("{mode_tag}{mode_value}");
        let right_w = right_plain.width();
        let start = width.saturating_sub(right_w) as u16;
        x >= start
    }
}

impl xpui::UiApp for DemoApp {
    fn set_window_size(&mut self, size: xpui::WindowSize) {
        self.window_size = size;
    }

    fn render(&mut self) -> xpui::Node {
        self.nav.focus.expire_quit_arm();
        let wrap_width = (self.window_size.width as usize).saturating_sub(2).max(1);
        let heights = self.chat.history_heights_memo.get_or_update(
            (self.chat.history.version(), wrap_width),
            || {
                self.chat
                    .history
                    .borrow()
                    .iter()
                    .map(|message| {
                        Self::wrapped_line_count(&Self::format_history_row(message, false), wrap_width)
                    })
                    .collect::<Vec<_>>()
            },
        );
        self.nav.list.set_item_heights(heights);
        self.nav
            .list_binding
            .sync_list_from_focus(&self.nav.focus, &mut self.nav.list);

        let mut should_scroll_to_bottom = false;
        self.history_events.drain(|event| {
            if matches!(event, HistoryEvent::UserAppended) {
                should_scroll_to_bottom = true;
            }
        });
        if should_scroll_to_bottom {
            let count = self.nav.list.item_count();
            if count > 0 {
                self.nav.list.set_focused_index(count - 1);
            }
        }

        let input_focused = self.is_input_focused();
        let input_container_focused = self.is_input_container_focused();
        let scroll_focused = self.is_scroll_focused();
        let dynamic_input_max = ((self.window_size.height * 0.20).floor() as u16).max(5);
        let input_wrap_width = (self.window_size.width as usize).max(8);
        let (input_visual_lines, _) = self.input_visual_metrics(input_wrap_width);
        let input_viewport_lines = input_visual_lines.clamp(1, dynamic_input_max);
        let max_input_offset = input_visual_lines.saturating_sub(input_viewport_lines);
        let input_offset_lines = self.input_scroll_offset.min(max_input_offset);
        let terminal_lines = (self.window_size.height as u16).max(1);
        // input(1 block) + help(2) + status(1) + vertical gaps(3)
        let reserved_without_history = 6u16.saturating_add(input_viewport_lines);
        let history_viewport_lines = terminal_lines.saturating_sub(reserved_without_history).max(3);
        self.nav.list.set_viewport_lines(history_viewport_lines);
        if should_scroll_to_bottom {
            self.nav.list.scroll_to_bottom();
        }

        let focused = self
            .nav
            .list_binding
            .focused_index(&self.nav.focus, self.nav.list.item_count());

        let mut list = xpui::column().gap(Self::ITEM_GAP_LINES as u8);
        for (i, message) in self.chat.history.borrow().iter().enumerate() {
            let i = i as u16;
            let is_focused = focused == Some(i);
            let body = Self::format_history_row(message, is_focused);
            list = list.child(
                xpui::container(xpui::text(body))
                    .focus(self.nav.list_binding.focus_id(i)),
            );
        }

        xpui::container(
            xpui::column()
                .gap(1)
                .child(
                    xpui::container(
                        xpui::scroll_view(list)
                            .focus(xpui::FocusId(Self::SCROLL_ID))
                            .viewport_lines(history_viewport_lines)
                            .offset_lines(self.nav.list.scroll_offset()),
                    ),
                )
                .child(
                    xpui::container(
                        xpui::scroll_view(
                            xpui::text_input_from_state(&self.chat.input)
                                .placeholder("Find and fix issues.")
                                .focus(xpui::FocusId(Self::INPUT_ID))
                                .focused(input_focused)
                                .gutter_highlighted(input_focused || input_container_focused)
                                .visible_offset_lines(input_offset_lines),
                        )
                        .viewport_lines(input_viewport_lines)
                        .offset_lines(input_offset_lines),
                    )
                    .focus(xpui::FocusId(Self::INPUT_CONTAINER_ID)),
                )
                .child(
                    xpui::container(
                        xpui::scroll_view(self.bottom_bar_node(
                            self.window_size.width as usize,
                            input_focused,
                            input_container_focused,
                            scroll_focused,
                        ))
                        .viewport_lines(2),
                    )
                    .style(xpui::BoxStyle::default().text_color(xpui::rgb(0xc9d1d9))),
                )
                .child(
                    xpui::container(self.status_bar_node(self.window_size.width as usize))
                        .style(
                            xpui::BoxStyle::default()
                                .bg(xpui::rgb(0x161b22))
                                .text_color(xpui::rgb(0xa5b1c2)),
                        ),
                ),
        )
        .style(xpui::BoxStyle::default().text_color(xpui::rgb(0xe6edf3)))
        .into_node()
    }

    fn on_input(&mut self, event: xpui::UiInputEvent) {
        if let xpui::UiInputEvent::MouseDown { x, y } = event {
            if self.is_mode_click(x, y) {
                self.mode = self.mode.cycle();
                return;
            }

            let history_lines = self.history_viewport_lines();
            if y < history_lines
                && let Some(index) = self.history_item_at_row(y)
            {
                self.nav.list.set_focused_index(index);
                self.nav
                    .focus
                    .set_focused(self.nav.list_binding.focus_id(index));
                return;
            }

            let input_top = history_lines.saturating_add(1);
            let (content_width, input_viewport_lines, input_offset_lines, gutter_digits) =
                self.input_layout_for_click();
            let input_bottom = input_top.saturating_add(input_viewport_lines);
            if y >= input_top && y < input_bottom {
                self.nav.focus.set_focused(xpui::FocusId(Self::INPUT_ID));
                let local_row = y.saturating_sub(input_top) as usize;
                let visual_row = usize::from(input_offset_lines).saturating_add(local_row);
                let content_x = x.saturating_sub((gutter_digits + 3) as u16) as usize;
                self.chat
                    .input
                    .set_cursor_from_visual_position(visual_row, content_x, content_width);
                return;
            }

            return;
        }

        if matches!(event, xpui::UiInputEvent::Key(xpui::UiKeyInput::ShiftTab)) {
            self.mode = self.mode.cycle();
            return;
        }

        let line_count = self.chat.input.value().split('\n').count().max(1);
        let gutter_digits = line_count.to_string().len();
        let input_total_width = (self.window_size.width as usize).max(8);
        let input_content_width = input_total_width.saturating_sub(gutter_digits + 3).max(1);
        self.chat.input.set_soft_wrap_width(Some(input_content_width));

        if self.is_input_focused() {
            if let xpui::UiInputEvent::ScrollLines(lines) = event {
                let max_offset = self.input_max_scroll_offset();
                if lines < 0 {
                    self.input_scroll_offset = self.input_scroll_offset.saturating_sub(1);
                } else if lines > 0 {
                    self.input_scroll_offset = self.input_scroll_offset.saturating_add(1);
                }
                self.input_scroll_offset = self.input_scroll_offset.min(max_offset);
                return;
            }

            if matches!(event, xpui::UiInputEvent::Key(xpui::UiKeyInput::Submit)) {
                let _ = self.chat.submit_input();
                return;
            }

            let key = match event {
                xpui::UiInputEvent::Key(key) => Some(key),
                _ => None,
            };
            if self.chat.input.handle_input(event) {
                if matches!(
                    key,
                    Some(
                        xpui::UiKeyInput::Up
                            | xpui::UiKeyInput::Down
                            | xpui::UiKeyInput::Enter
                            | xpui::UiKeyInput::Home
                            | xpui::UiKeyInput::End
                    )
                ) {
                    let (_, cursor_line) =
                        self.input_visual_metrics((self.window_size.width as usize).max(8));
                    let viewport = self.input_viewport_lines();
                    let min_offset = cursor_line.saturating_add(1).saturating_sub(viewport);
                    let max_offset = cursor_line;
                    self.input_scroll_offset =
                        self.input_scroll_offset.clamp(min_offset, max_offset);
                }
                self.clamp_input_scroll_offset();
                return;
            }
        }
        let _ = self
            .nav
            .list_binding
            .handle_input(&mut self.nav.focus, &mut self.nav.list, event);
    }

    fn focus_state(&mut self) -> Option<&mut xpui::FocusState> {
        Some(&mut self.nav.focus)
    }

    fn on_focus_entries(&mut self, entries: &[xpui::FocusEntry]) {
        let _ = self.nav.list_binding.sync_preferred_child_for_parent(
            &mut self.nav.focus,
            &self.nav.list,
            xpui::FocusId(Self::SCROLL_ID),
            entries,
        );
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
