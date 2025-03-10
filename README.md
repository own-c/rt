# RT (name pending)

A Twitch frontend written in Rust using SvelteKit and Tauri.

Add users manually or by replacing `https` with `rt` in the url, watch streams in any of the available resolutions and view chat with 7tv and BetterTTV emotes support, WIP ad skipping.

<table>
    <tr>
        <td align="center">
            <img alt="zfg1 playing OOT Randomizer" src=".github/assets/screenshot-1.jpg" width="500">
            <em>
                <div>
                    <a href="https://www.twitch.tv/zfg1">zfg1</a> playing OOT. Chat on Copium for Prime 4.
                </div>
            </em>
        </td>
        <td align="center">
            <img alt="paganmars playing Monster Hunter Wilds" src=".github/assets/screenshot-2.jpg" width="500">
            <em>
                <div>
                    <a href="https://www.twitch.tv/paganmars">paganmars</a> playing Monster Hunter Wilds. An excelent use of screen space with PiP.
                </div>
            </em>
        </td>
    </tr>
</table>

> This is not meant to be a replacement for the official Twitch app/site, some features (account login, send chat messages, etc) are not implemented and not in the scope of this project.

## About

Artifacts are uploaded [here](https://github.com/Kyagara/rt/actions) on successful builds, only bundles (installers) are built at the moment.

> Tested on Windows, other desktops should work but haven't been tested.

- Frontend `src`: [SvelteKit](https://svelte.dev/docs/kit/introduction), [tailwindcss](https://tailwindcss.com) and [Vidstack](https://github.com/vidstack/player).
- Backend `src-tauri`: [Tauri](https://tauri.app/).

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
- Try rt on other platforms.
- Use a shared public .env for some settings in both frontend and backend.
- Somehow make proxy work with `invoke`/another method that doesn't require a crate like axum (to avoid having a web server running).
- Service worker to cache emotes? Not sure if Tauri supports it (doesn't clear them on launch) or if theres a Tauri equivalent.
- Add back ability to rewind the video, currently using `ll-live` for the player which disables it.
- Use Tauri's channels to notify more events, like when the stream has been switched to a backup one and more.
- Try to reduce as much as possible the amount of requests made to Twitch.
- Fix `join_chat` errors when hard refreshing the page while in a chat.
