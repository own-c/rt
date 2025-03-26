<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	import { invoke } from '@tauri-apps/api/core';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

	import { error } from '$lib/components/Notification.svelte';
	import Grid from '$lib/components/Grid.svelte';

	import { streamingFor } from '$lib';
	import { Platform } from '$lib';

	let feed = $state([]) as LiveNow[];

	let loading = $state(false);

	async function updateView() {
		loading = true;

		try {
			await invoke<Feed>('get_feed', { platform: Platform.Twitch }).then((data) => {
				feed = data.twitch!.sort((a, b) => a.username.localeCompare(b.username));
			});
		} catch (err) {
			error('Error retrieving Twitch feed', err as string);
		}

		loading = false;
	}

	async function handleMouseWheelClick(event: MouseEvent, username: string) {
		// Middle mouse button
		if (event.button === 1) {
			try {
				await invoke('open_new_window', { url: `/streams/watch?username=${username}` });
			} catch (err) {
				error('Error opening new window', err as string);
			}
		}
	}

	onMount(async () => {
		const appWebview = getCurrentWebviewWindow();
		appWebview.listen<string>('updated_streams', async () => {
			await updateView();
		});

		await updateView();
	});
</script>

<div data-simplebar class="flex h-full w-full gap-2 p-2">
	{#if !loading && feed.length === 0}
		<span class="text-lg font-medium">No streams found</span>
	{:else}
		<Grid>
			{#each feed as live_now, index (index)}
				<button
					onmousedown={async (event: MouseEvent) =>
						await handleMouseWheelClick(event, live_now.username)}
					onclick={() => goto(`/streams/watch?username=${live_now.username}`)}
					class="flex cursor-pointer flex-col text-left hover:bg-neutral-800"
				>
					<img
						src={`https://static-cdn.jtvnw.net/previews-ttv/live_user_${live_now.username}-440x248.jpg`}
						alt={`Stream thumbnail for ${live_now.username}`}
					/>

					<div class="flex flex-col p-1">
						<span class="text-lg font-bold">{live_now.username}</span>

						<span class="text-sm text-neutral-400">{streamingFor(live_now.started_at)}</span>
					</div>
				</button>
			{/each}
		</Grid>
	{/if}
</div>
