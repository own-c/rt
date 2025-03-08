<script lang="ts">
	import 'vidstack/bundle';

	let { username, isLive, url } = $props();
</script>

{#key username}
	{#if isLive}
		<media-player
			autoPlay={true}
			streamType="ll-live"
			class="max-h-[calc(100vh-2rem)] max-w-[calc(100vw-2rem)]"
			style="--plyr-border-radius: 0px;"
		>
			<media-provider>
				<source src={url} type="application/x-mpegurl" />
			</media-provider>

			<media-plyr-layout
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
