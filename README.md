# somars

A terminal UI player for [SomaFM](https://somafm.com) internet radio, written in Rust.

```
 Stations (60)          Station Info
┌──────────────────────┬──────────────────────────────────────────┐
│ > Beat Blender       │ Title:       Groove Salad                │
│   Boot Liquor        │ Genre:       ambient                     │
│   Covers             │ DJ:          SomaFM                      │
│   Deep Space One     │ Listeners:   1899                        │
│   DEF CON Radio      │ Now Playing: Artist - Track Name         │
│   Drone Zone         │ Quality:     highest (MP3)               │
│ ♪ Groove Salad       │                                          │
│   Indie Pop Rocks!   │ Description:                             │
│   Secret Agent       │ A nicely chilled plate of ambient/       │
│   ...                │ downtempo beats and grooves.             │
└──────────────────────┴──────────────────────────────────────────┘
▶ Playing: Groove Salad     [↑↓/jk] Nav  [Enter] Play  [p] Pause  [s] Stop  [q] Quit
```

## Requirements

- [mpv](https://mpv.io/) — used for audio decoding and playback (routes through PipeWire automatically on modern Linux)
- Network access to `api.somafm.com`

Install mpv on Arch Linux:
```
sudo pacman -S mpv
```

## Build & Run

```bash
cargo build --release
./target/release/somars

# or
cargo run --release
```

## Key Bindings

| Key | Action |
|-----|--------|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `Enter` | Play selected station |
| `p` | Pause / resume |
| `s` | Stop playback |
| `q` | Quit |

## Architecture

```
src/
├── main.rs     — entry point: mpv check, channel fetch, TUI event loop
├── api.rs      — HTTP fetch + JSON deserialization of channels.json
├── app.rs      — application state and input actions
├── player.rs   — mpv subprocess management + IPC socket control
└── ui.rs       — ratatui rendering (station list + detail panel + status bar)
```

**Flow:**
1. On startup, `api::fetch_channels()` fetches `https://api.somafm.com/channels.json` synchronously and sorts stations alphabetically.
2. The ratatui TUI is initialized with a crossterm backend (alternate screen, raw mode).
3. The event loop polls for keystrokes every 100 ms and calls `app.tick()` to detect if mpv has exited.
4. On `Enter`, `player::Player::play()` spawns `mpv` with `--no-video` and `--input-ipc-server` pointing to a per-process UNIX socket at `/tmp/somars-<pid>.sock`.
5. Pause/resume sends JSON commands to the mpv IPC socket (`set_property pause true/false`).
6. Stop kills the mpv child process and removes the socket file.

## API Schema

Station data comes from the unofficial SomaFM JSON endpoint:

```
GET https://api.somafm.com/channels.json
```

Response shape:
```json
{
  "channels": [
    {
      "id": "groovesalad",
      "title": "Groove Salad",
      "description": "A nicely chilled plate of ambient/downtempo beats and grooves.",
      "dj": "SomaFM",
      "djmail": "dj@somafm.com",
      "genre": "ambient",
      "listeners": 1899,
      "lastPlaying": "Artist Name - Track Title",
      "image": "https://somafm.com/img3/groovesalad-400.jpg",
      "largeimage": "https://somafm.com/img3/groovesalad-400.jpg",
      "xlimage": "https://somafm.com/img3/groovesalad-1500.jpg",
      "updated": "1234567890",
      "playlists": [
        {
          "url": "https://somafm.com/groovesalad256.pls",
          "format": "mp3",
          "quality": "highest"
        },
        {
          "url": "https://somafm.com/groovesalad.pls",
          "format": "mp3",
          "quality": "high"
        },
        {
          "url": "https://somafm.com/groovesalad32.pls",
          "format": "aac",
          "quality": "low"
        }
      ]
    }
  ]
}
```

Stream quality values in priority order: `highest` → `high` → `low`. The `playlists[].url` is a `.pls` playlist file pointing to the actual stream URL; mpv handles `.pls` files natively.

> **Note:** This endpoint is unofficial. SomaFM [discontinued](https://somafm.com/linktous/api.html) third-party API access. It works as of mid-2026 but may change without notice. Consider supporting SomaFM at [somafm.com](https://somafm.com).
