mod app;
mod config;
mod hardware;
mod history;
mod i18n;
mod metrics;
mod presentmon;
mod steam;
mod system;
mod theme;
mod ui;

fn main() -> std::io::Result<()> {
    app::run()
}
