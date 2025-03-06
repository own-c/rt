# RT (name pending)

A Twitch frontend written in Rust using SvelteKit and Tauri.

<table>
    <tr>
        <td align="center">
            <img alt="zfg1 playing OOT Randomizer" src=".github/assets/screenshot-1.jpg" width="500"><br>
            <a href="https://www.twitch.tv/zfg1">
                <em>zfg1 playing OOT. Chat on Copium for Prime 4.</em>
            </a>
        </td>
        <td align="center">
            <img alt="paganmars playing Monster Hunter Wilds" src=".github/assets/screenshot-2.jpg" width="500"><br>
            <a href="https://www.twitch.tv/paganmars">
                <em>paganmars playing Monster Hunter Wilds. An excelent use of screen space with PiP.</em>
            </a>
        </td>
    </tr>
</table>

Add users manually or by replacing `https` with `rt` in the url, watch streams in any of the available resolutions and view chat with 7tv and BetterTTV emotes support.

> This is not meant to be a replacement for the official Twitch app/site, some features (account login, send chat messages, etc) are not implemented and not in the scope of this project.

## About

Artifacts are uploaded [here](https://github.com/Kyagara/rt/actions) on successful builds, only bundles (installers) are built at the moment.

> Tested on Windows, other desktops should work but haven't been tested.

- Frontend `src`: [SvelteKit](https://svelte.dev/docs/kit/introduction). Using [tailwindcss](https://tailwindcss.com) and [Vidstack](https://github.com/vidstack/player).
- Backend `src-tauri`: [Rust](https://www.rust-lang.org/), [Tauri](https://tauri.app/). Using [axum](https://github.com/tokio-rs/axum).

Data is stored in the following locations:

- Windows: `%AppData%/Roaming/com.rt.app`
- Linux: `~/.config/com.rt.app`

Logs are stored in the following locations:

- Linux: `$XDG_DATA_HOME/com.rt.app/logs` or `$HOME/.local/share/com.rt.app/logs`
- Windows: `%LocalAppData%/com.rt.app/logs`

## TODO

- Add global Twitch emotes and fetch the user emotes.
- Add persistent settings.
- Better error handling in the frontend (show notification when user is not found, etc).
- Button to show current stream info (game, viewcount, etc).
- Try rt on other platforms.
- Use a shared public .env for some settings in both frontend and backend.
- Add GitHub Actions to build (with `--no-bundle` for now) and provide artifacts for some platforms.
- Move config, data and logs to a single location.
- Somehow make proxy work with `invoke`/another method that doesn't require a crate like axum (to avoid having a web server running).
- Service worker to cache emotes? Not sure if Tauri supports it (doesn't clear them on launch) or if theres a Tauri equivalent.
