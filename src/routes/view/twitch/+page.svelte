<script lang="ts">
	import { onMount } from 'svelte';

	import { invoke } from '@tauri-apps/api/core';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

	import { error } from '$lib/components/Notification.svelte';

	import { streamingFor } from '$lib/Utils';

	let feed = $state([]) as LiveNow[];

	async function updateView() {
		try {
			await invoke<Feed>('get_feed', { platform: 'twitch' }).then((data) => {
				feed = data.twitch!;
			});
		} catch (err) {
			error('Error retrieving Twitch feed', err as string);
		}
	}

	onMount(async () => {
		const appWebview = getCurrentWebviewWindow();
		appWebview.listen<string>('update_view', async () => {
			await updateView();
		});

		await updateView();
	});
</script>

<div class="flex flex-col w-full h-full gap-2">
	{#if feed.length === 0}
		<div class="flex flex-col items-center justify-center">
			<span class="text-lg font-medium">No streams found</span>
		</div>
	{:else}
		<div class="flex w-full h-full p-2">
			{#each feed as live_now, index (index)}
				<div>
					<a
						href={`/watch/twitch?username=${live_now.username}`}
						class="flex flex-col items-center hover:bg-neutral-800 rounded-md cursor-pointer"
					>
						<img
							src={`https://static-cdn.jtvnw.net/previews-ttv/live_user_${live_now.username}-440x248.jpg`}
							alt="Stream thumbnail"
							class="aspect-16/9 object-contain max-h-32"
						/>

						<div class="flex flex-col justify-around w-full p-1">
							<span class="text-lg font-bold">{live_now.username}</span>
							<span class="text-sm text-neutral-400">{streamingFor(live_now.started_at)}</span>
						</div>
					</a>
				</div>
			{/each}
		</div>
	{/if}
</div>
