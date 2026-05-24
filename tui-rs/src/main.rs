mod app;
mod config;
mod doctor;
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
    if std::env::args()
        .skip(1)
        .any(|arg| matches!(arg.as_str(), "doctor" | "--doctor"))
    {
        return doctor::run();
    }

    app::run()
}
