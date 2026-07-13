# hippo

TUI application for browsing and streaming movies and TV series.

```
>=<                >=<
   ,.--'  ''-.        ,.--'  ''-.
   (  )  ',_.'        (  )  ',_.'
    Xx'xX              mn'mn`
```

## Features

- Browse trending movies and TV shows (Now Playing, Popular, Top Rated)
- Search movies and TV series via TMDB
- View season and episode details
- Open streaming links in your browser
- Keyboard-driven navigation (vim-style)

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/pr4j3sh/hippo/main/install.sh | bash
```

## Build from source

```bash
git clone https://github.com/pr4j3sh/hippo.git
cd hippo
cargo build --release
```

The binary will be at `target/release/hippo`.

## Usage

Set your TMDB API key:

```bash
export TMDB_API_KEY="your_key_here"
```

Get a free key at [themoviedb.org](https://www.themoviedb.org/settings/api).

Run:

```bash
hippo
```

### Keybindings

| Key | Action |
|-----|--------|
| `h` `j` `k` `l` / arrows | Navigate |
| `Space` / `Enter` | Select |
| `/` | Search |
| `Tab` | Toggle movie/TV search |
| `q` / `Esc` | Quit / Go back |
| `Ctrl+C` | Quit |

## License

MIT
