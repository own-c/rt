<script lang="ts">
	let { children } = $props();
</script>

<div class="container mx-auto">
	<div class="grid gap-2 grid-cols-[repeat(auto-fill,minmax(150px,1fr))]">
		{@render children()}
	</div>
</div>
