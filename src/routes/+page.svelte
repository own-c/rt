<script lang="ts">
	import { onMount } from 'svelte';

	import { getCurrent } from '@tauri-apps/plugin-deep-link';

	import { changeView } from '$lib/state/View.svelte';
	import { error } from '$lib/components/Notification.svelte';

	const twitchReg = /(?:https?:\/\/)?(?:www\.)?twitch\.tv\/([a-zA-Z0-9_]+)/;
	const youtubeReg =
		/(?:https?:\/\/)?(?:www\.)?(?:youtube\.com\/(?:[^/]+\/.+\/|(?:v|embed|shorts|watch)?\??v=|.*[?&]v=)|youtu\.be\/)([^"&?/\s]{11})/;

	async function handleURL(url: string) {
		if (!url) {
			error('No URL provided', '');
			return;
		}

		if (url.startsWith('rt://tw/') || url.startsWith('rt://twitch/')) {
			const username = url.replace(/^rt:\/\/(tw|twitch)\//, '').trim();
			if (!username) {
				error('Username was empty when opening via URL', url);
				return;
			}

			changeView('streams', true, `/watch?username=${username}`);
			return;
		}

		let matches = url.match(twitchReg);
		if (matches && matches[1]) {
			const username = matches[1];

			changeView('streams', true, `/watch?username=${username}`);
			return;
		}

		if (url.startsWith('rt://yt/') || url.startsWith('rt://youtube/')) {
			const videoId = url.replace(/^rt:\/\/(yt|youtube)\//, '').trim();
			if (!videoId) {
				error('Video ID was empty when opening via URL', url);
				return;
			}

			changeView('videos', true, `/watch?id=${videoId}`);
			return;
		}

		matches = url.match(youtubeReg);
		if (matches && matches[1]) {
			const videoId = matches[1];

			changeView('videos', true, `/watch?id=${videoId}`);
			return;
		}

		error('No matching URL found', url);
	}

	onMount(async () => {
		try {
			const current = await getCurrent();
			if (current && current[0]) {
				await handleURL(current[0]);
			} else {
				changeView(localStorage.getItem('lastView') || 'videos');
			}
		} catch (err) {
			error('Error handling URL', err as string);
		}
	});
</script>
