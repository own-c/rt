<script lang="ts">
	import { onMount } from 'svelte';

	import 'vidstack/bundle';

	import { changeView } from '$lib/state/View.svelte';

	let videoID = $state('');

	onMount(() => {
		const routeURL = new URL(window.location.href);
		videoID = routeURL.searchParams.get('id')!;
		changeView('videos', false);
	});
</script>

<div class="flex h-full w-full">
	<media-player
		storage="player-settings"
		src={`https://youtu.be/${videoID}`}
		autoPlay={true}
		streamType="on-demand"
		class="max-h-[calc(100vh-2rem)] max-w-[calc(100vw-2rem)]"
		style="--plyr-border-radius: 0px;"
	>
		<media-provider></media-provider>
		<media-plyr-layout></media-plyr-layout>
	</media-player>
</div>
