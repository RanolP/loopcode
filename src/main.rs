use clap::Parser;
use xpui::IntoNode;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long, help = "Run with graphics backend (gpui)")]
    graphics: bool,
}

struct DemoApp;

impl xpui::UiApp for DemoApp {
    fn render(&mut self) -> xpui::Node {
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
                ),
        )
        .style(
            xpui::BoxStyle::default()
                .bg(xpui::rgb(0x101418))
                .text_color(xpui::rgb(0xe6edf3)),
        )
        .into_node()
    }
}

fn main() {
    let args = Args::parse();

    if args.graphics {
        xpui::run_gpui(DemoApp);
    } else {
        xpui::run_cpui(DemoApp);
    }
}
