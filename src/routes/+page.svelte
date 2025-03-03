<script lang="ts">
	import { onMount } from 'svelte';

	import { onOpenUrl } from '@tauri-apps/plugin-deep-link';

	import Player from '$lib/components/Player.svelte';
	import Titlebar from '$lib/components/Titlebar.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import Chat from '$lib/components/Chat.svelte';

	import { initUsers } from '$lib/Users.svelte';
	import { switchStream, currentStream } from '$lib/Stream.svelte';

	let showChat = $state(false);
	function toggleChat() {
		showChat = !showChat;
	}

	onMount(async () => {
		await initUsers();

		await onOpenUrl(async (urls) => {
			const twitchRegex = new RegExp('twitch.tv/([a-zA-Z0-9_]+)');
			let username = '';

			if (urls && urls[0]) {
				const url = urls[0];

				const matches = url.match(twitchRegex);
				if (matches && matches[1]) {
					username = matches[1];
				} else {
					const parts = url.replace('rt://', '').trim().split('/');
					username = parts[0];
				}

				await switchStream(username);
			}
		});
	});
</script>

<div class="flex flex-col h-screen w-screen overflow-hidden bg-black text-white">
	<Titlebar {toggleChat} />

	<div class="flex min-h-full w-full">
		<Sidebar />

		<main class="flex w-full h-full">
			<div class="flex w-full h-full">
				{#if currentStream.url}
					<Player />
				{:else}
					<div class="flex flex-col items-center justify-center h-full w-full">
						<div class="text-center">
							<h1 class="text-4xl font-bold">No stream selected</h1>
							<p class="text-lg">Select a stream from the sidebar</p>
						</div>
					</div>
				{/if}
			</div>

			<div class="min-w-1/5 max-w-1/5 h-full" hidden={!showChat}>
				<Chat />
			</div>
		</main>
	</div>
</div>
