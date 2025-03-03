<script lang="ts">
	import 'vidstack/bundle';

	import { currentStream } from '$lib/Stream.svelte';
</script>

{#key currentStream.url || currentStream.live}
	{#if currentStream.live}
		<media-player
			autoplay
			stream-type="on-demand"
			style="--plyr-border-radius: 0px; max-height: calc(100vh - 2rem); max-width: calc(100vw - 2rem);"
		>
			<media-provider>
				<source src={currentStream.url} type="application/x-mpegurl" />
			</media-provider>

			<media-plyr-layout
				seek-time={5}
				display-duration={true}
				controls={[
					'play-large',
					'play',
					'progress',
					'current-time',
					'mute+volume',
					'settings',
					'pip',
					'airplay',
					'fullscreen'
				]}
			></media-plyr-layout>
		</media-player>
	{:else}
		<div class="flex flex-col items-center justify-center h-full w-full">
			<div class="text-center">
				<h1 class="text-4xl font-bold">No stream</h1>
			</div>
		</div>
	{/if}
{/key}

<style>
	:global(media-player video) {
		max-height: 100%;
		max-width: 100%;
	}
</style>
