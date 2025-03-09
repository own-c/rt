<script lang="ts">
	import { onMount } from 'svelte';

	import { getCurrentWindow } from '@tauri-apps/api/window';

	import { watching } from '$lib/logic/Stream.svelte';

	let { toggleChat } = $props();
	let showChat = $state(false);

	const appWindow = getCurrentWindow();
	let maximized = $state(false);

	function updateChatToggle() {
		showChat = !showChat;
		toggleChat();
	}

	onMount(async () => {
		await getCurrentWindow().onResized(async () => {
			maximized = await appWindow.isMaximized();
		});
	});
</script>

<header data-tauri-drag-region class="flex w-full bg-violet-800 min-h-8">
	<div class="w-12 min-w-12"></div>

	{#if watching.username}
		<div class="flex flex-1 justify-center gap-2">
			<div class="text-lg font-bold">
				{watching.username}
			</div>

			<div class="flex">
				<button title="Open in browser" class="px-2 hover:bg-neutral-700">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						width="1.2em"
						height="1.2em"
						viewBox="0 0 2048 2048"
						><!-- Icon from Fluent UI MDL2 by Microsoft Corporation - https://github.com/microsoft/fluentui/blob/master/packages/react-icons-mdl2/LICENSE --><path
							fill="currentColor"
							d="M1536 256h384v384h-128V475l-456 456l-91-91l456-456h-165zm0 768l128-128v768H0V512h1280l-128 128H128v896h1408z"
						/></svg
					>
				</button>

				<div class="px-2 hover:bg-neutral-700 flex items-center">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						width="1.2em"
						height="1.2em"
						viewBox="0 0 2048 2048"
						><!-- Icon from Fluent UI MDL2 by Microsoft Corporation - https://github.com/microsoft/fluentui/blob/master/packages/react-icons-mdl2/LICENSE --><path
							fill="currentColor"
							d="M960 1920q-133 0-255-34t-230-96t-194-150t-150-195t-97-229T0 960q0-133 34-255t96-230t150-194t195-150t229-97T960 0q133 0 255 34t230 96t194 150t150 195t97 229t34 256q0 133-34 255t-96 230t-150 194t-195 150t-229 97t-256 34m0-1792q-115 0-221 30t-198 84t-169 130t-130 168t-84 199t-30 221q0 114 30 220t84 199t130 169t168 130t199 84t221 30t220-30t199-84t169-130t130-168t84-199t30-221t-30-220t-84-199t-130-169t-168-130t-199-84t-221-30m-64 640h128v640H896zm0-256h128v128H896z"
						/></svg
					>
				</div>
			</div>
		</div>
	{:else}
		<div class="flex-1"></div>
	{/if}

	<div class="flex h-full">
		{#if watching.live}
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
