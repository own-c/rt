<script lang="ts">
	import { onMount } from 'svelte';

	import { onOpenUrl } from '@tauri-apps/plugin-deep-link';

	import Player from '$lib/components/Player.svelte';
	import Titlebar from '$lib/components/Titlebar.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import Chat from '$lib/components/Chat.svelte';

	import { initUsers } from '$lib/logic/Users.svelte';
	import { watching, fetchAndSetUser } from '$lib/logic/Stream.svelte';

	let showChat = $state(false);
	function toggleChat() {
		showChat = !showChat;
	}

	const twitchReg = new RegExp('twitch.tv/([a-zA-Z0-9_]+)');

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Tab') {
			event.preventDefault();
		}
	}

	onMount(async () => {
		await initUsers();

		await onOpenUrl(async (urls) => {
			let username = '';

			if (urls && urls[0]) {
				const url = urls[0];

				const matches = url.match(twitchReg);
				if (matches && matches[1]) {
					username = matches[1];
				} else {
					const parts = url.replace('rt://', '').trim().split('/');
					username = parts[0];
				}

				await fetchAndSetUser(username);
			}
		});
	});
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="flex flex-col h-screen w-screen overflow-hidden bg-black text-white"
	onkeydown={handleKeyDown}
>
	<Titlebar {toggleChat} />

	<div class="flex min-h-full w-full">
		<Sidebar />

		<main class="flex w-full h-full">
			<div class="flex w-full h-full">
				{#if watching.username}
					<Player username={watching.username} isLive={watching.live} url={watching.url} />
				{:else}
					<div class="flex flex-col items-center justify-center h-full w-full">
						<div class="text-center">
							<h1 class="text-4xl font-bold">No stream selected</h1>
							<p class="text-lg">Select a stream from the sidebar</p>
						</div>
					</div>
				{/if}
			</div>

			{#key watching.username}
				<div class="min-w-1/5 max-w-1/5 h-full" hidden={!showChat}>
					<Chat username={watching.username} isLive={watching.live} />
				</div>
			{/key}
		</main>
	</div>
</div>

<style>
	:global(html) {
		user-select: none;
		-webkit-user-select: none;
		-ms-user-select: none;
		outline: none;
	}
</style>
