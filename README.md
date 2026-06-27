# somars

A terminal UI player for [SomaFM](https://somafm.com) internet radio, written in Rust.

```
 Stations (3/60)        Station Info
┌──────────────────────┬──────────────────────────────────────────┐
│ / groove█            │ Title:       Groove Salad                │
│ > Groove Salad       │ Genre:       ambient                     │
│   Groove Salad 2     │ DJ:          SomaFM                      │
│   Groove Salad Clssc │ Listeners:   1899                        │
│                      │ Now Playing: Artist - Track Name         │
│                      │ Quality:     highest (MP3)               │
│                      │                                          │
│                      │ Description:                             │
│                      │ A nicely chilled plate of ambient/       │
│                      │ downtempo beats and grooves.             │
└──────────────────────┴──────────────────────────────────────────┘
▶ Playing: Groove Salad  ██████████  80%  [↑↓] Nav  [↵] Play+Exit  [Esc] Clear  [type] Filter
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

### Normal mode

| Key | Action |
|-----|--------|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `Enter` | Play selected station |
| `p` | Pause / resume |
| `s` | Stop playback |
| `←` / `h` | Volume down 5% |
| `→` / `l` | Volume up 5% |
| `/` | Enter search/filter mode |
| `q` | Quit |

### Filter mode (after pressing `/`)

| Key | Action |
|-----|--------|
| type | Narrow station list (matches title, genre, description) |
| `↑` / `↓` | Navigate filtered results |
| `Enter` | Play highlighted station and exit filter |
| `Esc` | Clear filter and return to full list |

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
5. Pause/resume and volume changes send JSON commands to the mpv IPC socket (`set_property pause`/`volume`).
6. Stop kills the mpv child process and removes the socket file.
7. Filter mode computes `filtered_indices()` on every keypress — a `Vec<usize>` of indices into the full channel list that match the query against title, genre, and description. The `ListState` cursor indexes into this filtered view; `now_playing_idx` always refers to the full list so the playing indicator survives filter changes.

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
