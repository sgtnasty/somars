mod api;
mod app;
mod player;
mod ui;

use app::App;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    check_mpv()?;

    println!("Fetching SomaFM stations...");
    let channels = api::fetch_channels()?;

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, channels);
    ratatui::restore();

    if let Err(ref e) = result {
        eprintln!("Error: {}", e);
    }
    result
}

fn run(
    terminal: &mut ratatui::DefaultTerminal,
    channels: Vec<api::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(channels);

    loop {
        terminal.draw(|f| ui::render(f, &mut app))?;
        app.tick();

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.prev(),
                    KeyCode::Enter => app.play_selected(),
                    KeyCode::Char('p') => app.toggle_pause(),
                    KeyCode::Char('s') => app.stop(),
                    KeyCode::Right | KeyCode::Char('l') => app.volume_up(),
                    KeyCode::Left | KeyCode::Char('h') => app.volume_down(),
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn check_mpv() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::{Command, Stdio};
    let ok = Command::new("mpv")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok {
        return Err("mpv not found — please install mpv to use somars".into());
    }
    Ok(())
}
