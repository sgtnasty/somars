# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build and run (release mode recommended — debug is noticeably slower for TUI)
cargo run --release

# Build only
cargo build --release

# Check for compile errors without building
cargo check

# Run tests
cargo test
```

There is no linter configured beyond `cargo check`. There are no integration or end-to-end tests; the app requires a live network connection and `mpv` to function.

## Architecture

Single-binary Rust TUI app. Five source files, no crates of their own:

- **`api.rs`** — blocking HTTP fetch of `https://api.somafm.com/channels.json` via `reqwest`; deserializes into `Channel` structs (note: `listeners` field uses a custom deserializer because the API returns it as either an integer or a numeric string). Returns channels sorted alphabetically by title.
- **`app.rs`** — owns all application state (`App` struct). Filter logic lives entirely here: `filtered_indices()` returns a `Vec<usize>` into the full `channels` vec on every call (no caching). `list_state` (a ratatui `ListState`) always indexes into the *filtered* view; `now_playing_idx` always indexes into the *full* `channels` vec so the playing indicator survives filter changes.
- **`player.rs`** — wraps an `mpv` child process. Spawns mpv with `--input-ipc-server=/tmp/somars-<pid>.sock`; all playback control (pause/resume, volume) goes through that UNIX socket via JSON IPC commands. `check_alive()` is called every tick to detect when mpv exits on its own.
- **`ui.rs`** — pure rendering with ratatui. Layout: 35/65 horizontal split (station list / station detail) above a 1-row status bar. The status bar renders a 10-block volume bar and context-sensitive key hints.
- **`main.rs`** — entry point: checks mpv exists, fetches channels, runs the ratatui event loop (100 ms poll interval).

## Key design constraints

- **No async** — the HTTP fetch at startup is synchronous (`reqwest` blocking feature). The TUI loop is blocking with a 100 ms `event::poll` timeout.
- **mpv is required** — the app hard-fails on startup if `mpv` is not in `$PATH`. All audio decoding and playback is delegated to mpv; the app only controls it via IPC.
- **IPC socket path** is `/tmp/somars-<pid>.sock` — one socket per process, cleaned up on `Player::stop()` and on `Drop`.
- **Stream selection** — `Channel::best_stream_url()` picks the highest-quality `.pls` playlist URL (highest → high → low). mpv handles `.pls` natively.
- **SomaFM API is unofficial** — the endpoint at `api.somafm.com/channels.json` is not officially supported and may break without notice.
