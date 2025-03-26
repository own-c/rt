<script lang="ts">
	import { onMount } from 'svelte';

	import { invoke } from '@tauri-apps/api/core';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
	import { open } from '@tauri-apps/plugin-dialog';

	import { error, info } from '$lib/components/Notification.svelte';
	import Grid from '$lib/components/Grid.svelte';

	import { getAvatarUrl } from '$lib';
	import { Platform } from '$lib';

	let users = $state([]) as User[];
	let loading = $state(false);

	let filter = $state(Platform.Twitch);
	let channelName = $state('');

	async function addUser(username: string) {
		loading = true;

		if (!username) {
			info('No username provided');
			return;
		}

		try {
			await invoke('add_user', { username, platform: filter });
		} catch (err) {
			error(`Error adding user '${username}'`, err as string);
			return;
		}

		loading = false;
		info(`Added '${username}'`);
	}

	async function updateUser(username: string, platform: Platform) {
		try {
			await invoke('add_user', { username, platform });
		} catch (err) {
			error(`Error updating user '${username}'`, err as string);
			return;
		}

		info(`Updated '${username}'`);
	}

	async function removeUser(username: string, platform: Platform) {
		try {
			await invoke('remove_user', { username, platform });
		} catch (err) {
			error(`Error removing user '${username}'`, err as string);
			return;
		}

		info(`Removed '${username}'`);
	}

	async function importSubscriptions() {
		const subscriptionsFilePath = await open({
			multiple: false,
			directory: false,
			filters: [{ name: 'CSV', extensions: ['csv'] }]
		});

		try {
			const data = await invoke<number>('import_subscriptions', { subscriptionsFilePath });

			info(`Imported ${data} subscriptions`);
		} catch (err) {
			error('Error importing subscriptions', err as string);
			return;
		}

		await updateView();
	}

	async function updateView() {
		try {
			await invoke<User[]>('get_users').then((data) => {
				users = data.sort((a, b) => a.username.localeCompare(b.username));
			});
		} catch (err) {
			error('Error retrieving users', err as string);
		}
	}

	onMount(async () => {
		const appWebview = getCurrentWebviewWindow();
		appWebview.listen<string>('updated_users', async () => {
			await updateView();
		});

		loading = true;
		await updateView();
		loading = false;
	});
</script>

<div class="flex flex-col h-full w-full p-2 gap-3">
	<div class="flex gap-2 items-center mx-4">
		<select
			bind:value={filter}
			class="py-1 px-2 border border-gray-600 bg-gray-800 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
		>
			<option value={Platform.Twitch} class="bg-gray-800">Twitch</option>
			<option value={Platform.YouTube} class="bg-gray-800">YouTube</option>
		</select>

		<hr class="border-gray-700 h-full mx-1" />

		<form onsubmit={async () => await addUser(channelName)} class="flex gap-2 items-center">
			<span class="font-medium">Add user:</span>

			<input
				type="text"
				bind:value={channelName}
				placeholder="Channel name"
				class="px-3 py-1 border border-gray-600 bg-gray-800 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
			/>
		</form>

		{#if filter === Platform.YouTube}
			<button
				onclick={async () => await importSubscriptions()}
				class="bg-gray-800 py-1 px-4 border border-gray-600 rounded-md hover:outline-none hover:ring-2 hover:ring-blue-500 cursor-pointer"
			>
				Import subscriptions
			</button>
		{/if}
	</div>

	<hr class="border-gray-700 w-full" />

	<div class="flex w-full">
		{#if !loading && users.filter((user) => user.platform === filter).length === 0}
			<span class="text-lg font-medium">No users found</span>
		{:else}
			<Grid>
				{#each users as user, index (index)}
					{#if user.platform === filter}
						<div class="flex flex-col items-center">
							<img
								src={getAvatarUrl(user.avatar)}
								id={user.username}
								alt={'Avatar of ' + user.username}
								class="w-16 h-16 rounded-full"
							/>

							<div class="flex flex-col w-full items-center justify-between">
								<span class="text-lg font-medium">{user.username}</span>

								<div class="flex w-full">
									<button
										disabled={loading}
										title={filter === Platform.Twitch
											? 'Update emotes and avatar'
											: 'Update avatar'}
										class="block w-full px-2 py-1 bg-neutral-500 hover:bg-neutral-600 cursor-pointer"
										onclick={async () => await updateUser(user.username, user.platform)}
									>
										Update
									</button>

									<button
										disabled={loading}
										title="Remove user"
										class="block px-2 py-1 bg-red-500 hover:bg-red-600 cursor-pointer"
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
			</Grid>
		{/if}
	</div>
</div>
