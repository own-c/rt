<script lang="ts">
	import { fade } from 'svelte/transition';

	let { children } = $props();
</script>

<div class="container mx-auto">
	<div
		class="grid gap-2 grid-cols-[repeat(auto-fill,minmax(150px,1fr))]"
		in:fade={{ duration: 200 }}
	>
		{@render children()}
	</div>
</div>
