<script lang="ts">
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';

	import { onOpenUrl } from '@tauri-apps/plugin-deep-link';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

	import 'simplebar';
	import 'simplebar/dist/simplebar.css';

	import Player from '$lib/components/Player.svelte';
	import Titlebar from '$lib/components/Titlebar.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import Chat from '$lib/components/Chat.svelte';
	import Notification, { error, info } from '$lib/components/Notification.svelte';

	import { watching, initStores, updateUser } from '$lib/Stores.svelte';

	const appWebview = getCurrentWebviewWindow();
	appWebview.listen<string>('stream', (event) => {
		switch (event.payload) {
			case 'main':
				info('No ads detected, switching main stream.');
				break;
			case 'backup':
				info('Found Ads, switching to backup stream.');
				break;
		}
	});

	let showChat = $state(false);
	let movingMouse = $state(false);
	let timer = $state(0);

	function toggleChat() {
		showChat = !showChat;
	}

	function handleMousemove() {
		movingMouse = true;

		clearTimeout(timer);

		timer = setTimeout(() => {
			movingMouse = false;
		}, 2000);
	}

	const twitchReg = new RegExp('twitch.tv/([a-zA-Z0-9_]+)');

	// Disable tab navigation
	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Tab') {
			event.preventDefault();
		}
	}

	onMount(async () => {
		await initStores();

		await onOpenUrl(async (urls) => {
			if (urls && urls[0]) {
				const url = urls[0];

				let username = '';

				const matches = url.match(twitchReg);
				if (matches && matches[1]) {
					username = matches[1];
				} else {
					const parts = url.replace('rt://', '').trim().split('/');
					username = parts[0];
				}

				if (!username) {
					error(`Username was empty when opening via URL`, `URLS: ${urls.join(', ')}`);
					return;
				}

				await updateUser(username, true);
			}
		});
	});
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="flex flex-col h-screen w-screen overflow-hidden bg-black text-white"
	onkeydown={handleKeyDown}
>
	{#key watching.username}
		<Titlebar />
	{/key}

	<main class="flex min-h-full w-full items-center justify-center">
		<Sidebar />

		<div class="flex w-full h-full" onmousemove={handleMousemove}>
			{#key watching.username}
				<div class="flex w-full h-full">
					{#if watching.username}
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

				{#if watching.url}
					{#if movingMouse && !showChat}
						<button
							title="Expand chat"
							class="fixed top-8 right-0 p-2 z-50 hover:bg-neutral-700"
							onclick={toggleChat}
							in:fade={{ duration: 25 }}
							out:fade={{ duration: 200 }}
						>
							<svg
								xmlns="http://www.w3.org/2000/svg"
								width="1em"
								height="1em"
								viewBox="0 0 2048 2048"
								><!-- Icon from Fluent UI MDL2 by Microsoft Corporation - https://github.com/microsoft/fluentui/blob/master/packages/react-icons-mdl2/LICENSE --><path
									fill="currentColor"
									d="m1170 146l-879 878l879 878l-121 121l-999-999l999-999zm853 0l-878 878l878 878l-121 121l-999-999l999-999z"
								/></svg
							>
						</button>
					{/if}

					{#if showChat}
						<button
							title="Expand chat"
							class="fixed top-8 right-0 p-2 z-50 hover:bg-neutral-700"
							onclick={toggleChat}
						>
							<svg
								xmlns="http://www.w3.org/2000/svg"
								width="1em"
								height="1em"
								viewBox="0 0 2048 2048"
								><!-- Icon from Fluent UI MDL2 by Microsoft Corporation - https://github.com/microsoft/fluentui/blob/master/packages/react-icons-mdl2/LICENSE --><path
									fill="currentColor"
									d="m903 146l879 878l-879 878l121 121l999-999l-999-999zm-853 0l878 878l-878 878l121 121l999-999L171 25z"
								/></svg
							>
						</button>
					{/if}
				{/if}

				<div class="h-full min-w-1/5 max-w-1/5" hidden={!showChat}>
					<Chat />
				</div>
			{/key}
		</div>
	</main>

	<Notification />
</div>

<style>
	:global(html) {
		user-select: none;
		-webkit-user-select: none;
		-ms-user-select: none;
		outline: none;
	}

	:global(.simplebar-scrollbar) {
		transition: opacity 0.2s ease-in-out;
	}

	:global(.simplebar-scrollbar::before) {
		background-color: #ffffff;
	}
</style>
