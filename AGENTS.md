# hippo

TUI application for browsing and streaming movies/TV series.

## Tech stack

- **Language**: Rust (edition 2024)
- **TUI framework**: Ratatui
- **TMDB API**: Movie/TV metadata and search. Auth via `TMDB_API_KEY` env var.
- **Vidsrc API**: Streaming source (see patterns below)

## Build & run

```bash
cargo build
cargo run
```

No tests, lints, or typecheck targets configured yet.

## Environment

`TMDB_API_KEY` env var is **required** — app exits immediately if missing.

## Architecture

Single crate, 5 source files:

- `src/main.rs` — entrypoint, event loop, key handling
- `src/app.rs` — application state, async API calls via `tokio::spawn` + `mpsc` channel
- `src/tmdb.rs` — TMDB API client
- `src/ui.rs` — Ratatui rendering
- `src/logging.rs` — `fern`-based file logging

**Async pattern**: API calls run in `tokio::spawn` tasks, results sent back via unbounded `mpsc` channel as `AppAction` enum variants. The main loop polls the receiver each tick.

## Vidsrc streaming URLs

- Movies: `https://vidsrcme.ru/embed/movie?tmdb={TMDB_ID}`
- TV Series: `https://vidsrcme.ru/embed/tv?tmdb={TMDB_ID}&season={S}&episode={E}`

**Current approach**: URLs open in the system browser via the `open` crate. No in-terminal playback.

## Key bindings

- `q` / `Esc` — quit or go back
- `hjkl` / arrow keys — navigate
- `/` — enter search mode
- `Tab` — toggle movie/TV search
- `Enter` / `Space` — select item
