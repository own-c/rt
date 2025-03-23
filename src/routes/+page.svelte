<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	import { getCurrent } from '@tauri-apps/plugin-deep-link';

	import { changeView } from '$lib/state/View.svelte';
	import { error } from '$lib/components/Notification.svelte';

	const twitchReg = new RegExp('twitch.tv/([a-zA-Z0-9_]+)');

	async function handleURL(url: string) {
		let username = '';

		const matches = url.match(twitchReg);
		if (matches && matches[1]) {
			username = matches[1];
		} else {
			const parts = url
				.replace(/^rt:\/+/, '')
				.trim()
				.split('/');

			username = parts[0];
		}

		if (!username) {
			error(`Username was empty when opening via URL`, `URL: ${url}`);
			return;
		}

		goto(`/streams/watch?username=${username}`);
	}

	onMount(async () => {
		await getCurrent().then(async (current) => {
			if (current && current[0]) {
				await handleURL(current[0]);
			} else {
				changeView('streams');
			}
		});
	});
</script>
