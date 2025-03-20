<script lang="ts">
	import { onMount, tick } from 'svelte';

	import { info } from './Notification.svelte';

	import {
		live_now,
		users,
		watching,
		refreshUsers,
		removeUser,
		updateUser,
		parseTime,
		joinStream,
		is_ui_loading
	} from '$lib/Stores.svelte';

	let rightClickedUser = $state('');

	let inputEl = $state() as HTMLInputElement;
	let username = $state('');
	let showInput = $state(false);

	async function toggleInput() {
		showInput = !showInput;
		username = '';

		await tick();
		if (inputEl) inputEl.focus();
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape' && (showContextMenu || showInput)) {
			showInput = false;
			showContextMenu = false;
		}
	}

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

	async function handleRefreshUsers() {
		await refreshUsers();
		info('Refreshed users');
	}

	function getStreamingFor(username: string) {
		const liveNow = live_now[username];
		if (!liveNow) return username;

		const diff = new Date().getTime() - new Date(liveNow.started_at).getTime();
		const totalSeconds = Math.floor(diff / 1000);
		const time = parseTime(totalSeconds);

		return `${username} - ${time}`;
	}

	async function changeStream(username: string) {
		showInput = false;
		await joinStream(username);
	}

	async function handleAddUser(username: string) {
		showInput = false;
		await updateUser(username, true);
		info(`Added '${username}'`);
	}

	async function handleUpdateUser() {
		showContextMenu = false;
		await updateUser(rightClickedUser, false);
		info(`Updated '${rightClickedUser}'`);
	}

	async function handleRemoveUser() {
		showContextMenu = false;
		await removeUser(rightClickedUser);
		info(`Removed '${rightClickedUser}'`);
	}

	onMount(() => {
		document.addEventListener('click', handleLeftClick);
		document.addEventListener('keydown', handleKeyDown);
	});
</script>

<aside class="flex flex-col items-center h-full min-w-12 bg-neutral-800 gap-2 user-select-none">
	<div class="flex flex-col items-center w-full">
		<button
			aria-label="Add user"
			title="Add user"
			onclick={toggleInput}
			class="flex flex-col items-center cursor-pointer hover:bg-neutral-600 w-full py-2"
		>
			<svg xmlns="http://www.w3.org/2000/svg" width="1.5em" height="1.5em" viewBox="0 0 2048 2048"
				><path
					fill="currentColor"
					d="M1024 0q141 0 272 36t244 104t207 160t161 207t103 245t37 272q0 141-36 272t-104 244t-160 207t-207 161t-245 103t-272 37q-141 0-272-36t-245-103t-207-160t-160-208t-103-244t-37-273q0-141 36-272t104-244t160-207t207-161T752 37t272-37m0 1920q124 0 238-32t214-90t181-140t140-181t91-214t32-239t-32-238t-90-214t-140-181t-181-140t-214-91t-239-32t-238 32t-214 90t-181 140t-140 181t-91 214t-32 239t32 238t90 214t140 182t181 140t214 90t239 32m64-961h448v128h-448v449H960v-449H512V959h448V512h128z"
				/></svg
			>
		</button>

		<button
			aria-label="Refresh users"
			title="Refresh users"
			disabled={is_ui_loading()}
			onclick={async () => await handleRefreshUsers()}
			class="flex flex-col items-center cursor-pointer w-full py-2 {is_ui_loading()
				? 'opacity-50'
				: 'hover:bg-neutral-600'}"
		>
			<svg xmlns="http://www.w3.org/2000/svg" width="1.5em" height="1.5em" viewBox="0 0 2048 2048"
				><path
					fill="currentColor"
					d="M1297 38q166 45 304 140t237 226t155 289t55 331q0 141-36 272t-103 245t-160 207t-208 160t-245 103t-272 37q-141 0-272-36t-245-103t-207-160t-160-208t-103-244t-37-273q0-140 37-272t105-248t167-212t221-164H256V0h512v512H640V215q-117 56-211 140T267 545T164 773t-36 251q0 123 32 237t90 214t141 182t181 140t214 91t238 32q123 0 237-32t214-90t182-141t140-181t91-214t32-238q0-150-48-289t-136-253t-207-197t-266-124z"
				/></svg
			>
		</button>
	</div>

	<hr class="border-gray-700 w-full" />

	<div data-simplebar class="w-full h-full overflow-y-auto pb-8">
		{#each Object.values(users).sort((a, b) => {
			const aLive = live_now[a.username] ? 1 : 0;
			const bLive = live_now[b.username] ? 1 : 0;
			return bLive - aLive;
		}) as user, index (index)}
			{@const disabled =
				!users ||
				Object.keys(users).length === 0 ||
				is_ui_loading() ||
				user.username === watching.username}

			<button
				id={user.username}
				title={live_now[user.username] ? getStreamingFor(user.username) : user.username}
				{disabled}
				class="flex flex-col items-center w-full py-1 {disabled
					? 'opacity-50'
					: 'cursor-pointer hover:bg-neutral-600'}"
				onclick={async () => await changeStream(user.username)}
				oncontextmenu={handleContextMenu}
			>
				<div class="relative">
					{#if !user.avatar}
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
							src={user.avatar}
							id={user.username}
							alt={'Avatar of ' + user.username}
							class="rounded-full w-10 h-10"
						/>
					{/if}

					{#if live_now[user.username]}
						<span
							class="absolute top-0 right-0 h-3 w-3 bg-red-600 rounded-full border-1 border-black shadow-md"
						></span>
					{/if}
				</div>
			</button>
		{/each}
	</div>
</aside>

{#if showInput}
	<form onsubmit={async () => await handleAddUser(username)}>
		<input
			bind:this={inputEl}
			bind:value={username}
			type="text"
			placeholder="Channel name"
			spellcheck="false"
			autocomplete="on"
			class="fixed top-10 left-[60px] px-2 shadow-md w-32 z-50 bg-gray-800 border border-white rounded-md outline-none user-select outline"
		/>
	</form>
{/if}

{#if showContextMenu}
	<div
		bind:this={contextMenuEl}
		class="flex flex-col gap-1 absolute shadow-lg rounded z-50 bg-neutral-700 py-1"
		style="top: {rightClickPos.y}px; left: {rightClickPos.x + 10}px;"
	>
		<button
			class="hover:bg-neutral-500 px-2 cursor-pointer w-full"
			onclick={async () => await handleUpdateUser()}
		>
			Update
		</button>

		<button
			class="hover:bg-red-500 px-2 cursor-pointer w-full"
			onclick={async () => await handleRemoveUser()}
		>
			Remove
		</button>
	</div>
{/if}
