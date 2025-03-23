<script lang="ts">
	import { onMount } from 'svelte';

	import { invoke } from '@tauri-apps/api/core';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

	import { error, info } from '$lib/components/Notification.svelte';

	import { getAvatarUrl } from '$lib/Utils';

	let users = $state([]) as User[];

	let filter = $state('twitch');

	let loading = $state(false);

	async function updateUser(username: string, platform: Platform) {
		try {
			await invoke('add_user', { username: username, platform: platform });
		} catch (err) {
			error(`Error updating user '${username}'`, err as string);
			return;
		}

		info(`Updated '${username}'`);
	}

	async function removeUser(username: string, platform: Platform) {
		try {
			await invoke('remove_user', { username: username, platform: platform });
		} catch (err) {
			error(`Error removing user '${username}'`, err as string);
			return;
		}

		info(`Removed '${username}'`);
	}

	async function updateView() {
		try {
			await invoke<User[]>('get_users').then((data) => {
				users = data.sort((a, b) => a.username.localeCompare(b.username));
			});
		} catch (err) {
			error('Error retrieving Twitch users', err as string);
		}
	}

	onMount(async () => {
		const appWebview = getCurrentWebviewWindow();
		appWebview.listen<string>('update_view', async () => {
			await updateView();
		});

		loading = true;
		await updateView();
		loading = false;
	});
</script>

<div class="flex flex-col h-full w-full p-2 gap-2">
	<div>
		<select bind:value={filter}>
			<option value="twitch" class="bg-neutral-900">Twitch</option>
			<option value="youtube" class="bg-neutral-900">YouTube</option>
		</select>
	</div>

	<hr class="border-gray-700 w-full" />

	<div class="flex gap-2 w-full">
		{#if loading}
			<span class="text-lg font-medium">Loading...</span>
		{:else if users.length === 0 || users.filter((user) => user.platform === filter).length === 0}
			<span class="text-lg font-medium">No users found</span>
		{:else}
			{#each users as user, index (index)}
				{#if user.platform === filter}
					<div class="flex flex-col items-center">
						<img
							src={getAvatarUrl(user.avatarBlob)}
							id={user.username}
							alt={'Avatar of ' + user.username}
							class="w-16 h-16 rounded-full"
						/>

						<div class="flex flex-col w-full items-center justify-between">
							<span class="text-lg font-medium">{user.username}</span>

							<div class="flex">
								<button
									disabled={loading}
									title={filter === 'twitch' ? 'Update emotes and avatar' : ''}
									class="block w-full px-2 py-1 bg-neutral-500 hover:bg-neutral-600"
									onclick={async () => await updateUser(user.username, user.platform)}
								>
									Update
								</button>

								<button
									disabled={loading}
									title="Remove user"
									class="block w-full px-2 py-1 bg-red-500 hover:bg-red-600"
									onclick={async () => await removeUser(user.username, user.platform)}
								>
									<svg
										xmlns="http://www.w3.org/2000/svg"
										width="1em"
										height="1em"
										viewBox="0 0 2048 2048"
										><!-- Icon from Fluent UI MDL2 by Microsoft Corporation - https://github.com/microsoft/fluentui/blob/master/packages/react-icons-mdl2/LICENSE --><path
											fill="currentColor"
											d="M1792 384h-128v1472q0 40-15 75t-41 61t-61 41t-75 15H448q-40 0-75-15t-61-41t-41-61t-15-75V384H128V256h512V128q0-27 10-50t27-40t41-28t50-10h384q27 0 50 10t40 27t28 41t10 50v128h512zM768 256h384V128H768zm768 128H384v1472q0 26 19 45t45 19h1024q26 0 45-19t19-45zM768 1664H640V640h128zm256 0H896V640h128zm256 0h-128V640h128z"
										/></svg
									>
								</button>
							</div>
						</div>
					</div>
				{/if}
			{/each}
		{/if}
	</div>
</div>
