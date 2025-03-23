<script lang="ts">
	import { onMount } from 'svelte';

	import { changeView } from '$lib/state/View.svelte';

	onMount(() => {
		changeView('twitch');
	});
</script>
