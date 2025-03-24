<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	import { invoke } from '@tauri-apps/api/core';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

	import { error } from '$lib/components/Notification.svelte';
	import Grid from '$lib/components/Grid.svelte';

	import { Platform, timeAgo } from '$lib';

	let feed = $state([]) as YouTubeVideo[];

	let loading = $state(false);

	async function updateView() {
		loading = true;

		try {
			await invoke<Feed>('get_feed', { platform: Platform.YouTube }).then((data) => {
				feed = data.youtube!.sort((a, b) => {
					return Number(b.publish_date) - Number(a.publish_date);
				});
			});
		} catch (err) {
			error('Error retrieving YouTube feed', err as string);
		}

		loading = false;
	}

	async function handleMouseWheelClick(event: MouseEvent, videoID: string) {
		// Middle mouse button
		if (event.button === 1) {
			try {
				await invoke('open_new_window', { url: `/videos/watch?id=${videoID}` });
			} catch (err) {
				error('Error opening new window', err as string);
			}
		}
	}

	onMount(async () => {
		const appWebview = getCurrentWebviewWindow();
		appWebview.listen<string>('updated_videos', async () => {
			await updateView();
		});

		await updateView();
	});
</script>

<div data-simplebar class="flex w-full h-full gap-2 p-2">
	{#if loading}
		<span class="text-lg font-medium">Loading...</span>
	{:else if feed.length === 0}
		<span class="text-lg font-medium">No videos found</span>
	{:else}
		<div class="w-full h-full">
			<Grid>
				{#each feed as video, index (index)}
					<button
						onmousedown={async (event: MouseEvent) => await handleMouseWheelClick(event, video.id)}
						onclick={async () => goto(`/videos/watch?id=${video.id}`)}
						class="flex flex-col hover:bg-neutral-800 cursor-pointer text-left"
					>
						<img
							src={`https://i.ytimg.com/vi/${video.id}/mqdefault.jpg`}
							alt={`Video thumbnail for ${video.id}`}
						/>

						<span title={video.title} class="text-md font-semibold text-elipsis">
							{video.title}
						</span>

						<span class="text-xs">
							{video.username}
							{video.view_count ? `- ${video.view_count} views` : ''} - {timeAgo(
								video.publish_date
							)}
						</span>
					</button>
				{/each}
			</Grid>
		</div>
	{/if}
</div>
