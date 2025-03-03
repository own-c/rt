<script>
    import { onMount } from "svelte";

    import { onOpenUrl } from "@tauri-apps/plugin-deep-link";
    import { fetch } from "@tauri-apps/plugin-http";

    import Player from "$lib/components/Player.svelte";
    import Titlebar from "$lib/components/Titlebar.svelte";
    import Sidebar from "$lib/components/Sidebar.svelte";
    import Chat from "$lib/components/Chat.svelte";

    import { loadUsers, setUser, users } from "$lib/Users.svelte";
    import { stream, setNewStream } from "$lib/Stream.svelte";
    import { addEmotes } from "$lib/Emotes.svelte";

    let showChat = $state(false);

    async function getStream(username) {
        switch (username.split("/").length) {
            case 2:
                username = username.split("/")[1];
                break;
            case 4:
                username = username.split("/")[3];
                break;
            default:
                username = username;
                break;
        }

        const response = await fetch("http://127.0.0.1:3030/user/" + username);

        if (response.status !== 200) {
            const parsed = await response.json();
            console.log("Error fetching", response.statusText, parsed);

            if (!users[username]) {
                let newUser = {
                    username: username,
                    live: false,
                };

                await setUser(newUser);
            }

            return;
        }

        const data = await response.json();

        await addEmotes(username, data.emotes);

        setNewStream(data);

        let newUser = {
            username: username,
            avatar: data.avatar,
            live: true,
        };

        await setUser(newUser);
    }

    function toggleChat() {
        showChat = !showChat;
    }

    onMount(async () => {
        await loadUsers();

        await onOpenUrl(async (urls) => {
            const twitchRegex = /www.twitch.tv\/([a-zA-Z0-9_]+)/;
            if (urls && urls[0] && twitchRegex.test(urls[0])) {
                await getStream(urls[0]);
            }
        });
    });
</script>

<div
    class="flex flex-col h-screen w-screen overflow-hidden bg-black text-white"
>
    <Titlebar {toggleChat} />

    <div class="flex min-h-full w-full">
        <Sidebar {getStream} />

        <main class="flex w-full h-full">
            <div class="flex w-full h-full">
                {#if stream.url}
                    <Player />
                {/if}
            </div>

            <div
                class="bg-secondary min-w-1/5 max-w-1/5 h-full"
                hidden={!showChat}
            >
                <Chat />
            </div>
        </main>
    </div>
</div>
