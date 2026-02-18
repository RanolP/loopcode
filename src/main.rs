use clap::Parser;
use xpui::IntoNode;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long, help = "Run with graphics backend (gpui)")]
    graphics: bool,
}

struct DemoApp {
    scroll_offset: u16,
}

impl DemoApp {
    fn new() -> Self {
        Self { scroll_offset: 0 }
    }
}

impl xpui::UiApp for DemoApp {
    fn render(&mut self) -> xpui::Node {
        const ITEM_COUNT: u16 = 24;
        const VIEWPORT_LINES: u16 = 8;
        let max_offset = ITEM_COUNT.saturating_sub(VIEWPORT_LINES);

        let mut log = xpui::column().gap(1);
        for i in 1..=ITEM_COUNT {
            log = log.child(xpui::text(format!("• Scroll item #{i:02}")));
        }

        xpui::container(
            xpui::column()
                .gap(2)
                .child(xpui::text("Hello from xpui").run(" (bold)", xpui::TextStyle::new().bold()))
                .child(
                    xpui::text("Styled: ")
                        .run("italic", xpui::TextStyle::new().italic())
                        .run(
                            " + underline",
                            xpui::TextStyle::new()
                                .underline()
                                .color(xpui::rgb(0x6dd3fb)),
                        ),
                )
                .child(xpui::text(format!(
                    "ScrollView demo (↑/↓, PgUp/PgDn, Home/End, wheel) offset={}/{}",
                    self.scroll_offset, max_offset
                )))
                .child(
                    xpui::scroll_view(log)
                        .viewport_lines(VIEWPORT_LINES)
                        .offset_lines(self.scroll_offset),
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
        const ITEM_COUNT: u16 = 24;
        const VIEWPORT_LINES: u16 = 8;
        let max_offset = ITEM_COUNT.saturating_sub(VIEWPORT_LINES);

        match event {
            xpui::UiInputEvent::Key(xpui::UiKeyInput::Up) => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            xpui::UiInputEvent::Key(xpui::UiKeyInput::Down) => {
                self.scroll_offset = self.scroll_offset.saturating_add(1).min(max_offset);
            }
            xpui::UiInputEvent::Key(xpui::UiKeyInput::PageUp) => {
                self.scroll_offset = self.scroll_offset.saturating_sub(VIEWPORT_LINES);
            }
            xpui::UiInputEvent::Key(xpui::UiKeyInput::PageDown) => {
                self.scroll_offset = self
                    .scroll_offset
                    .saturating_add(VIEWPORT_LINES)
                    .min(max_offset);
            }
            xpui::UiInputEvent::Key(xpui::UiKeyInput::Home) => {
                self.scroll_offset = 0;
            }
            xpui::UiInputEvent::Key(xpui::UiKeyInput::End) => {
                self.scroll_offset = max_offset;
            }
            xpui::UiInputEvent::ScrollLines(lines) if lines < 0 => {
                let delta = lines.unsigned_abs();
                self.scroll_offset = self.scroll_offset.saturating_sub(delta);
            }
            xpui::UiInputEvent::ScrollLines(lines) if lines > 0 => {
                let delta = lines as u16;
                self.scroll_offset = self.scroll_offset.saturating_add(delta).min(max_offset);
            }
            _ => {}
        }
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
