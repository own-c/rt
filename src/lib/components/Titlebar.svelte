<script lang="ts">
	import { onMount } from 'svelte';

	import { getCurrentWindow } from '@tauri-apps/api/window';

	import { currentStream } from '$lib/Stream.svelte';

	let { toggleChat } = $props();
	let showChat = $state(false);

	const appWindow = getCurrentWindow();
	let maximized = $state(false);

	function updateChatToggle() {
		showChat = !showChat;
		toggleChat();
	}

	$effect(() => {
		if (currentStream.username && currentStream.title) {
			appWindow.setTitle(currentStream.username + ': ' + currentStream.title);
		}
	});

	onMount(async () => {
		await getCurrentWindow().onResized(async () => {
			maximized = await appWindow.isMaximized();
		});
	});
</script>

<header data-tauri-drag-region class="flex w-full bg-violet-800 min-h-8">
	{#if currentStream.username && currentStream.title}
		<div title={currentStream.title} class="flex-1 text-center text-lg font-bold">
			{currentStream.username}
		</div>
	{:else}
		<div class="flex-1"></div>
	{/if}

	<div class="flex h-full">
		{#if currentStream.url}
			<button
				aria-label="Expand chat"
				title={showChat ? 'Collapse chat' : 'Expand chat'}
				onclick={() => updateChatToggle()}
				class="px-2 hover:bg-neutral-700"
			>
				{#if showChat}
					<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 2048 2048"
						><path
							fill="currentColor"
							d="m903 146l879 878l-879 878l121 121l999-999l-999-999zm-853 0l878 878l-878 878l121 121l999-999L171 25z"
						/></svg
					>
				{:else}
					<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 2048 2048"
						><path
							fill="currentColor"
							d="m1170 146l-879 878l879 878l-121 121l-999-999l999-999zm853 0l-878 878l878 878l-121 121l-999-999l999-999z"
						/></svg
					>
				{/if}
			</button>
		{/if}

		<hr class="border-gray-700 h-full mx-2" />

		<button
			aria-label="Minimize"
			title="Minimize"
			onclick={() => appWindow.minimize()}
			class="px-2 hover:bg-neutral-700"
		>
			<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 2048 2048"
				><path fill="currentColor" d="M2048 819v205H0V819z" /></svg
			>
		</button>

		<button
			aria-label="Maximize"
			title={maximized ? 'Restore window' : 'Maximize window'}
			onclick={() => appWindow.toggleMaximize()}
			class="px-2 hover:bg-neutral-700"
		>
			{#if maximized}
				<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 2048 2048"
					><path fill="currentColor" d="M1024 1657L25 658l121-121l878 878l878-878l121 121z" /></svg
				>
			{:else}
				<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 2048 2048"
					><path
						fill="currentColor"
						d="m1902 1511l-878-878l-878 878l-121-121l999-999l999 999z"
					/></svg
				>
			{/if}
		</button>

		<button
			aria-label="Close"
			title="Close"
			onclick={() => appWindow.close()}
			class="px-2 hover:bg-red-600"
		>
			<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 2048 2048"
				><path
					fill="currentColor"
					d="m1169 1024l879 879l-145 145l-879-879l-879 879L0 1903l879-879L0 145L145 0l879 879L1903 0l145 145z"
				/></svg
			>
		</button>
	</div>
</header>

<style>
	header {
		-webkit-app-region: drag;
		user-select: none;
	}

	button {
		-webkit-app-region: no-drag;
	}
</style>
