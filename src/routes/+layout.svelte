<script lang="ts">
	import '../app.css';

	import 'simplebar';
	import 'simplebar/dist/simplebar.css';

	import Notification from '$lib/components/Notification.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import Titlebar from '$lib/components/Titlebar.svelte';

	let { children } = $props();
</script>

<div class="flex flex-col h-screen w-screen bg-neutral-950 text-white">
	<Titlebar />

	<div class="flex flex-1 min-h-0">
		<Sidebar />

		<main class="flex w-full h-full min-h-0 overflow-auto">
			{@render children()}
		</main>
	</div>

	<Notification />
</div>

<style>
	:global(html) {
		user-select: none;
		-webkit-user-select: none;
		-ms-user-select: none;
	}

	:global(.simplebar-scrollbar) {
		transition: opacity 0.2s ease-in-out;
	}

	:global(.simplebar-scrollbar::before) {
		background-color: #ffffff;
	}
</style>
