<script lang="ts">
	import 'vidstack/bundle';

	import { watching } from '$lib/Stores.svelte';
</script>

{#key watching.url}
	<media-player
		storage="player-settings"
		autoPlay={true}
		streamType="on-demand"
		class="max-h-[calc(100vh-2rem)] max-w-[calc(100vw-2rem)]"
		style="--plyr-border-radius: 0px;"
	>
		<media-provider>
			<source src={watching.url} type="application/x-mpegurl" />
		</media-provider>

		<media-plyr-layout
			displayDuration={true}
			controls={[
				'play-large',
				'play',
				'progress',
				'current-time',
				'mute+volume',
				'settings',
				'pip',
				'fullscreen'
			]}
		></media-plyr-layout>
	</media-player>
{/key}

<style>
	:global(media-player video) {
		max-height: 100%;
		max-width: 100%;
	}
</style>
