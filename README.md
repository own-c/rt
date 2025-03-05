# RT (name pending)

A Twitch frontend written in Rust using SvelteKit and Tauri.

Add users manually or by going to `rt://www.twitch.tv/echo_esports` (replacing `https` with `rt` in the url), watch streams in any of the available resolutions and view chat with 7tv and BetterTTV emotes support.

> This is not meant to be a replacement for the official Twitch app/site, some features (account login, send chat messages, etc) are not implemented and not in the scope of this project.

## Structure

Tested on Windows, other desktops should work but haven't been tested. I manually upload the latest working binary to the [releases](https://github.com/Kyagara/rt/releases) page, I don't name a version number currently so you need to check when the latest release is uploaded.

- Frontend `src`: [SvelteKit](https://svelte.dev/docs/kit/introduction). Using [tailwindcss](https://tailwindcss.com) and [Vidstack](https://github.com/vidstack/player).
- Backend `src-tauri`: [Rust](https://www.rust-lang.org/), [Tauri](https://tauri.app/). Using [axum](https://github.com/tokio-rs/axum).

Data is stored in the following locations:

- Windows: `%AppData%/Roaming/com.rt.app`
- Linux: `~/.config/com.rt.app`

Logs are stored in the following locations:

- Linux: `$XDG_DATA_HOME/com.rt.app/logs` or `$HOME/.local/share/com.rt.app/logs`
- Windows: `%LocalAppData%/com.rt.app/logs`

## Building

The project is build using `lld` linker, so you might need to install it before building:

```bash
cargo install -f cargo-binutils
rustup component add llvm-tools
```

## TODO

- Add global Twitch emotes and fetch the user emotes.
- Add persistent settings.
- Better error handling in the frontend (show notification when user is not found, etc).
- Button to show current stream info (game, viewcount, etc).
- Try rt on other platforms.
- Use a shared public .env for some settings in both frontend and backend.
- Add GitHub Actions to build (with `--no-bundle` for now) and provide artifacts for some platforms.
- Move config, data and logs to a single location.
- Handle chat messages (including regex/emotes) from the backend, which would return fragments that the frontend would just render.
- Somehow make both endpoints (proxy/chat) work with `invoke`/another method that doesn't require a crate like axum.
