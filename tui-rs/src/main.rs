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
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "doctor" | "--doctor"))
    {
        return doctor::run();
    }
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "rtss-dump" | "--rtss-dump"))
    {
        return frames::run_rtss_dump();
    }

    app::run()
}
