<script lang="ts">
	import { onMount } from 'svelte';

	import { getCurrentWindow } from '@tauri-apps/api/window';

	import { currentView } from '$lib/state/View.svelte';

	const appWindow = getCurrentWindow();
	let maximized = $state(false);

	onMount(async () => {
		await getCurrentWindow().onResized(async () => {
			maximized = await appWindow.isMaximized();
		});
	});
</script>

<header data-tauri-drag-region class="flex w-full min-h-8 bg-neutral-600">
	<span data-tauri-drag-region class="flex items-center px-2 text-lg font-medium"
		>{currentView.name}</span
	>

	<div data-tauri-drag-region class="flex-1"></div>

	<div class="flex h-full">
		<button
			aria-label="Minimize"
			title="Minimize"
			onclick={() => appWindow.minimize()}
			class="px-2 hover:bg-neutral-700"
		>
			<svg xmlns="http://www.w3.org/2000/svg" width="1rem" height="1rem" viewBox="0 0 2048 2048"
				><!-- Icon from Fluent UI MDL2 by Microsoft Corporation - https://github.com/microsoft/fluentui/blob/master/packages/react-icons-mdl2/LICENSE --><path
					fill="currentColor"
					d="M2048 819v205H0V819z"
				/></svg
			>
		</button>

		<button
			aria-label="Maximize"
			title={maximized ? 'Restore window' : 'Maximize window'}
			onclick={() => appWindow.toggleMaximize()}
			class="px-2 hover:bg-neutral-700"
		>
			{#if maximized}
				<svg xmlns="http://www.w3.org/2000/svg" width="1rem" height="1rem" viewBox="0 0 2048 2048"
					><!-- Icon from Fluent UI MDL2 by Microsoft Corporation - https://github.com/microsoft/fluentui/blob/master/packages/react-icons-mdl2/LICENSE --><path
						fill="currentColor"
						d="M1024 1657L25 658l121-121l878 878l878-878l121 121z"
					/></svg
				>
			{:else}
				<svg xmlns="http://www.w3.org/2000/svg" width="1rem" height="1rem" viewBox="0 0 2048 2048"
					><!-- Icon from Fluent UI MDL2 by Microsoft Corporation - https://github.com/microsoft/fluentui/blob/master/packages/react-icons-mdl2/LICENSE --><path
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
			class="px-2 hover:bg-red-500"
		>
			<svg xmlns="http://www.w3.org/2000/svg" width="1rem" height="1rem" viewBox="0 0 2048 2048"
				><!-- Icon from Fluent UI MDL2 by Microsoft Corporation - https://github.com/microsoft/fluentui/blob/master/packages/react-icons-mdl2/LICENSE --><path
					fill="currentColor"
					d="m1169 1024l879 879l-145 145l-879-879l-879 879L0 1903l879-879L0 145L145 0l879 879L1903 0l145 145z"
				/></svg
			>
		</button>
	</div>
</header>
