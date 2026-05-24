mod app;
mod config;
mod frames;
mod game_resolver;
mod hardware;
mod history;
mod hotkeys;
mod i18n;
mod metrics;
mod overlay;
mod steam;
mod system;
mod theme;
mod ui;

fn main() -> std::io::Result<()> {
    app::run()
}
