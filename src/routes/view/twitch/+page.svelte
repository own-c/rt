<script lang="ts">
	import { onMount } from 'svelte';

	import { invoke } from '@tauri-apps/api/core';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

	import { error, info } from '$lib/components/Notification.svelte';

	import { getAvatarUrl, streamingFor } from '$lib/Utils';

	let users = $state([]) as User[];

	let feed = $state([]) as LiveNow[];

	const appWebview = getCurrentWebviewWindow();
	appWebview.listen<string>('update_view', async () => {
		await updateView();
	});

	let rightClickedUser = $state('');

	let contextMenuEl = $state() as HTMLDivElement;
	let rightClickPos = $state({ x: 0, y: 0 });
	let showContextMenu = $state(false);

	function handleContextMenu(event: MouseEvent) {
		event.preventDefault();
		rightClickedUser = (event.target as HTMLElement).id;
		rightClickPos = { x: event.clientX, y: event.clientY };
		showContextMenu = true;
	}

	function handleLeftClick() {
		if (contextMenuEl && showContextMenu) {
			showContextMenu = false;
		}
	}

	async function updateUser() {
		showContextMenu = false;
		//await updateUser(rightClickedUser, false);
		info(`Updated '${rightClickedUser}'`);
	}

	async function removeUser() {
		showContextMenu = false;
		//await removeUser(rightClickedUser);
		info(`Removed '${rightClickedUser}'`);
	}

	async function updateView() {
		try {
			await invoke<User[]>('get_users', { platform: 'twitch' }).then((data) => {
				users = data;
			});
		} catch (err) {
			error('Error retrieving Twitch users', err as string);
		}

		try {
			await invoke<Feed>('get_feed', { platform: 'twitch' }).then((data) => {
				feed = data.twitch!;
			});
		} catch (err) {
			error('Error retrieving Twitch feed', err as string);
		}
	}

	onMount(async () => {
		document.addEventListener('click', handleLeftClick);
		await updateView();
	});
</script>

<div class="flex flex-col w-full h-full gap-2">
	<div data-simplebar data-simplebar-direction="rtl" class="flex w-full bg-neutral-900">
		{#each users as user, index (index)}
			<button
				id={user.username}
				class="flex flex-col items-center p-1 hover:bg-neutral-600 cursor-pointer"
				oncontextmenu={handleContextMenu}
			>
				<div>
					{#if !user.avatarBlob}
						<div class="flex items-center justify-center rounded-full w-10 h-10">
							<svg
								xmlns="http://www.w3.org/2000/svg"
								width="1.5em"
								height="1.5em"
								viewBox="0 0 2048 2048"
								><path
									fill="currentColor"
									d="m2048 1544l-512-256v248H0V512h1536v248l512-256zm-640-904H128v768h1280zm512 71l-384 193v240l384 193z"
								/></svg
							>
						</div>
					{:else}
						<img
							width={50}
							height={50}
							src={getAvatarUrl(user.avatarBlob)}
							id={user.username}
							alt={'Avatar of ' + user.username}
							class="rounded-full w-10 h-10"
						/>
					{/if}
				</div>
			</button>
		{/each}
	</div>

	<hr class="border-gray-700 w-full" />

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
						class="aspect-16/9 object-contain max-h-48"
					/>

					<div class="flex flex-col justify-around w-full p-1">
						<span class="text-lg font-bold">{live_now.username}</span>
						<span class="text-sm text-neutral-400">{streamingFor(live_now.started_at)}</span>
					</div>
				</a>
			</div>
		{/each}
	</div>
</div>

{#if showContextMenu}
	<div
		bind:this={contextMenuEl}
		class="flex flex-col gap-1 absolute shadow-lg rounded z-50 bg-neutral-700 py-1"
		style="top: {rightClickPos.y}px; left: {rightClickPos.x + 10}px;"
	>
		<button
			class="hover:bg-neutral-500 px-2 cursor-pointer w-full"
			onclick={async () => await updateUser()}
		>
			Update
		</button>

		<button
			class="hover:bg-red-500 px-2 cursor-pointer w-full"
			onclick={async () => await removeUser()}
		>
			Remove
		</button>
	</div>
{/if}
