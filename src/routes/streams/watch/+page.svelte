<script lang="ts">
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';

	import { invoke } from '@tauri-apps/api/core';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

	import Player from '$lib/components/Player.svelte';
	import Chat from '$lib/components/Chat.svelte';
	import { error, info } from '$lib/components/Notification.svelte';

	let windowLabel = $state('');
	let username = $state('');
	let url = $state('');

	let showChat = $state(false);
	let movingMouse = $state(false);

	let movingMouseTimer = $state(0);

	function toggleChat() {
		showChat = !showChat;
	}

	function handleMousemove() {
		movingMouse = true;

		clearTimeout(movingMouseTimer);

		movingMouseTimer = setTimeout(() => {
			movingMouse = false;
		}, 2000);
	}

	onMount(async () => {
		const appWebview = getCurrentWebviewWindow();
		windowLabel = appWebview.label;

		appWebview.listen<string>('stream', (event) => {
			switch (event.payload) {
				case 'main':
					info('No ads detected, switching main stream.');
					break;

				case 'backup':
					info('Found ads, switching to backup stream.');
					break;
			}
		});

		const routeURL = new URL(window.location.href);
		username = routeURL.searchParams.get('username')!;

		try {
			await invoke<string>('fetch_stream_playback', { username, backup: false }).then((data) => {
				url = data;
			});
		} catch (err) {
			error('Stream not found', err as string);
		}
	});
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="flex w-full h-full" onmousemove={handleMousemove}>
	{#if url}
		<Player {windowLabel} {username} {url} />
	{/if}
</div>

{#if movingMouse && !showChat}
	<button
		title="Expand chat"
		class="fixed top-8 right-0 p-2 z-50 hover:bg-neutral-700"
		onclick={toggleChat}
		in:fade={{ duration: 25 }}
		out:fade={{ duration: 200 }}
	>
		<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 2048 2048"
			><!-- Icon from Fluent UI MDL2 by Microsoft Corporation - https://github.com/microsoft/fluentui/blob/master/packages/react-icons-mdl2/LICENSE --><path
				fill="currentColor"
				d="m1170 146l-879 878l879 878l-121 121l-999-999l999-999zm853 0l-878 878l878 878l-121 121l-999-999l999-999z"
			/></svg
		>
	</button>
{/if}

{#if showChat}
	<button
		title="Hide chat"
		class="fixed top-8 right-0 p-2 z-50 hover:bg-neutral-700"
		onclick={toggleChat}
	>
		<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 2048 2048"
			><!-- Icon from Fluent UI MDL2 by Microsoft Corporation - https://github.com/microsoft/fluentui/blob/master/packages/react-icons-mdl2/LICENSE --><path
				fill="currentColor"
				d="m903 146l879 878l-879 878l121 121l999-999l-999-999zm-853 0l878 878l-878 878l121 121l999-999L171 25z"
			/></svg
		>
	</button>
{/if}

<div class="h-full min-w-1/5 max-w-1/5" hidden={!showChat}>
	{#if url}
		<Chat {username} />
	{/if}
</div>
