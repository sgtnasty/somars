use crate::app::{App, PlaybackState};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(outer[0]);

    render_station_list(f, app, body[0]);
    render_station_detail(f, app, body[1]);
    render_status_bar(f, app, outer[1]);
}

fn render_station_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .channels
        .iter()
        .enumerate()
        .map(|(i, ch)| {
            let is_playing = app.now_playing_idx == Some(i);
            let style = if is_playing {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(vec![Span::styled(ch.title.clone(), style)]))
        })
        .collect();

    let title = format!(" Stations ({}) ", app.channels.len());
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_station_detail(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Station Info ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(area);

    let content: Vec<Line> = if let Some(ch) = app.selected_channel().cloned() {
        let label = || Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Title:       ", label()),
                Span::styled(
                    ch.title.clone(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Genre:       ", label()),
                Span::styled(ch.genre.clone(), Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("DJ:          ", label()),
                Span::styled(ch.dj.clone(), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Listeners:   ", label()),
                Span::styled(
                    ch.listeners.to_string(),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];

        if let Some(ref np) = ch.last_playing {
            if !np.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("Now Playing: ", label()),
                    Span::styled(
                        np.clone(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]));
            }
        }

        if let Some(pl) = ch.best_playlist() {
            lines.push(Line::from(vec![
                Span::styled("Quality:     ", label()),
                Span::styled(
                    format!("{} ({})", pl.quality, pl.format.to_uppercase()),
                    Style::default().fg(Color::Magenta),
                ),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Description:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            ch.description.clone(),
            Style::default().fg(Color::Gray),
        )));

        if let Some(ref err) = app.error.clone() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                err.clone(),
                Style::default().fg(Color::Red),
            )));
        }

        lines
    } else {
        vec![Line::from(Span::styled(
            "No station selected",
            Style::default().fg(Color::DarkGray),
        ))]
    };

    f.render_widget(block, area);

    let paragraph = Paragraph::new(content).wrap(Wrap { trim: true });
    f.render_widget(paragraph, inner);
}

fn render_status_bar(f: &mut Frame, app: &mut App, area: Rect) {
    let (icon, color) = match app.playback_state() {
        PlaybackState::Playing => ("▶", Color::Green),
        PlaybackState::Paused => ("⏸", Color::Yellow),
        PlaybackState::Stopped => ("■", Color::DarkGray),
    };

    let msg = if app.status_msg.is_empty() {
        format!("{} Stopped", icon)
    } else {
        format!("{} {}", icon, app.status_msg)
    };

    let vol = app.player.volume;
    let filled = (vol as usize * 10 / 100).min(10);
    let vol_bar: String = format!(
        "{}{}",
        "█".repeat(filled),
        "░".repeat(10 - filled),
    );

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(18),
            Constraint::Length(62),
        ])
        .split(area);

    let status = Paragraph::new(Line::from(vec![Span::styled(
        format!(" {}", msg),
        Style::default().fg(color),
    )]));
    f.render_widget(status, chunks[0]);

    let vol_color = if vol >= 80 {
        Color::Yellow
    } else {
        Color::Cyan
    };
    let volume = Paragraph::new(Line::from(vec![
        Span::styled(vol_bar, Style::default().fg(vol_color)),
        Span::styled(
            format!(" {:3}%", vol),
            Style::default().fg(Color::White),
        ),
    ]));
    f.render_widget(volume, chunks[1]);

    let help = Paragraph::new(Line::from(Span::styled(
        "[↑↓/jk] Nav  [↵] Play  [p] Pause  [s] Stop  [←→/hl] Vol  [q] Quit",
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(help, chunks[2]);
}
