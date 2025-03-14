<script lang="ts" module>
	import { fly, fade } from 'svelte/transition';

	import { error as logError } from '@tauri-apps/plugin-log';

	let visible = $state(false);
	let notificationMessage = $state('');

	export function info(message: string) {
		notificationMessage = message;

		visible = true;

		setTimeout(() => {
			visible = false;
		}, 3000);
	}

	export function error(message: string, error: string) {
		notificationMessage = message;

		logError(`${message}: ${error}`);

		visible = true;

		setTimeout(() => {
			visible = false;
		}, 3000);
	}
</script>

{#if visible}
	<div
		role="alert"
		class="fixed bottom-4 left-1/2 transform -translate-x-1/2 rounded-lg shadow-lg bg-black/60 text-white text-center p-2 z-100"
		in:fly={{ y: 20, duration: 300 }}
		out:fade={{ duration: 300 }}
	>
		{notificationMessage}
	</div>
{/if}
