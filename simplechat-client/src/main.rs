use anyhow::Result;
use clap::Parser;

mod app;
mod components;
mod tui;

// This prevents the console from being messed up if we panic for some reason.
fn initialize_panic_handler() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the user
    #[arg(short, long, default_value = "Anonymous")]
    name: String,

    /// Remote server to connect to
    #[arg(short, long, default_value = "localhost:3000")]
    addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    initialize_panic_handler();
    let args = Args::parse();
    app::run(args.addr, args.name).await?;
    Ok(())
}
