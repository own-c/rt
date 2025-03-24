<script lang="ts">
	import 'vidstack/bundle';

	import { onMount } from 'svelte';

	let videoID = $state('');

	onMount(() => {
		const routeURL = new URL(window.location.href);
		videoID = routeURL.searchParams.get('id')!;
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

<style>
	:global(media-player video) {
		max-height: 100%;
		max-width: 100%;
	}
</style>
