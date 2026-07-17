# hippo

TUI app for browsing and streaming movies/TV series.

## Tech stack

- **Language**: Rust (edition 2024) — no tests, lints, or typecheck configured
- **TUI**: Ratatui 0.29 + crossterm 0.28
- **HTTP**: reqwest 0.12 with `rustls-tls`
- **Async runtime**: tokio (full features)
- **Logging**: fern → file at `{cache_dir}/hippo/hippo.log`

## Build & run

```bash
cargo build
cargo run
```

`TMDB_API_KEY` env var is **required** — app exits immediately if missing.

## Architecture

Single crate, 5 source files:

- `src/main.rs` — entrypoint, terminal setup, event loop, key dispatch
- `src/app.rs` — app state (`App`), async API calls, `AppAction` enum
- `src/tmdb.rs` — TMDB REST client (`TmdbClient`)
- `src/ui.rs` — Ratatui rendering
- `src/logging.rs` — `fern`-based file logging

**Async pattern**: API calls run in `tokio::spawn` tasks, results sent back via unbounded `mpsc` channel as `AppAction` variants. Main loop polls the receiver each tick.

## Vidsrc streaming

- Movies: `https://vidsrcme.ru/embed/movie?tmdb={TMDB_ID}`
- TV: `https://vidsrcme.ru/embed/tv?tmdb={TMDB_ID}&season={S}&episode={E}`
- URLs open in system browser via the `open` crate — no in-terminal playback.
