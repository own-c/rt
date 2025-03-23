<script lang="ts">
	import { tick } from 'svelte';

	import { error, info } from './Notification.svelte';
	import { invoke } from '@tauri-apps/api/core';

	import { currentView, changeView } from '$lib/state/View.svelte';

	let loading = $state(false);

	let inputEl = $state() as HTMLInputElement;
	let username = $state('');
	let showInput = $state(false);

	async function toggleUserInput() {
		showInput = !showInput;
		username = '';

		await tick();
		if (inputEl) inputEl.focus();
	}

	async function refreshFeed() {
		loading = true;

		try {
			await invoke('refresh_feed', { platform: currentView.id });
		} catch (err) {
			error(`Error refreshing ${currentView.name} feed`, err as string);
			return;
		}

		loading = false;
		info('Refreshed feed');
	}

	async function addUser(username: string) {
		showInput = false;
		loading = true;

		try {
			await invoke('add_user', { username: username, platform: currentView.id });
		} catch (err) {
			error(`Error adding user '${username}'`, err as string);
			return;
		}

		loading = false;
		info(`Added '${username}'`);
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape' && showInput) {
			showInput = false;
		}
	}
</script>

<aside
	class="flex flex-col items-center h-full w-12 min-w-12 bg-neutral-800 gap-2 user-select-none flex-shrink-0"
>
	<div class="flex flex-col items-center w-full">
		<button
			aria-label="Videos"
			title="Videos"
			onclick={() => changeView('youtube')}
			class="flex flex-col items-center cursor-pointer hover:bg-neutral-600 w-full py-2"
		>
			<svg xmlns="http://www.w3.org/2000/svg" width="1.5rem" height="1.5rem" viewBox="0 0 2048 2048"
				><!-- Icon from Fluent UI MDL2 by Microsoft Corporation - https://github.com/microsoft/fluentui/blob/master/packages/react-icons-mdl2/LICENSE --><path
					fill="currentColor"
					d="m2048 1544l-512-256v248H0V512h1536v248l512-256zm-640-904H128v768h1280zm512 71l-384 193v240l384 193z"
				/></svg
			>
		</button>

		<button
			aria-label="Streams"
			title="Streams"
			onclick={() => changeView('twitch')}
			class="flex flex-col items-center cursor-pointer hover:bg-neutral-600 w-full py-2"
		>
			<svg xmlns="http://www.w3.org/2000/svg" width="1.5rem" height="1.5rem" viewBox="0 0 2048 2048"
				><!-- Icon from Fluent UI MDL2 by Microsoft Corporation - https://github.com/microsoft/fluentui/blob/master/packages/react-icons-mdl2/LICENSE --><path
					fill="currentColor"
					d="M1024 779q51 0 95 19t78 53t52 77t20 96q0 51-19 95t-53 78t-77 52t-96 20q-51 0-95-19t-78-53t-52-77t-20-96q0-51 19-95t53-78t77-52t96-20m0 384q29 0 54-11t44-29t30-44t11-55t-11-54t-29-44t-44-30t-55-11t-54 11t-44 29t-30 44t-11 55t11 54t29 44t44 30t55 11m716-855q72 71 127 154t93 174t57 189t20 199q0 101-19 199t-58 189t-93 174t-127 154l-75-75q64-64 113-138t83-156t51-169t18-178q0-90-17-177t-51-170t-83-156t-114-138zM383 383q-64 64-113 138t-84 156t-51 169t-18 178q0 90 17 177t52 170t83 156t114 138l-75 75q-72-71-127-154t-93-174t-57-189t-20-199q0-101 19-199t58-189t93-174t127-154zm1086 196q89 90 136 204t48 241q0 126-47 240t-137 205l-75-75q74-74 113-169t40-201q0-105-39-200t-114-170zm-815 75q-74 74-113 169t-40 201q0 105 39 200t114 170l-75 75q-89-90-136-204t-48-241q0-126 47-240t137-205z"
				/></svg
			>
		</button>

		<button
			aria-label="Users"
			title="Users"
			onclick={() => changeView('users')}
			class="flex flex-col items-center cursor-pointer hover:bg-neutral-600 w-full py-2"
		>
			<svg xmlns="http://www.w3.org/2000/svg" width="1.5rem" height="1.5rem" viewBox="0 0 2048 2048"
				><!-- Icon from Fluent UI MDL2 by Microsoft Corporation - https://github.com/microsoft/fluentui/blob/master/packages/react-icons-mdl2/LICENSE --><path
					fill="currentColor"
					d="M1397 1550q-21-114-78-210t-141-166t-189-110t-221-40q-88 0-170 23t-153 64t-129 100t-100 130t-65 153t-23 170H0q0-117 35-229t101-207t157-169t203-113q-56-36-100-83t-76-103t-47-119t-17-129q0-106 40-199t109-163T568 40T768 0t199 40t163 109t110 163t40 200q0 66-16 129t-48 119t-75 103t-101 83q99 38 183 100t147 143t105 177t54 202l-57 58zM384 512q0 80 30 149t82 122t122 83t150 30q79 0 149-30t122-82t83-122t30-150q0-79-30-149t-82-122t-123-83t-149-30q-80 0-149 30t-122 82t-83 123t-30 149m1645 941l-557 558l-269-270l90-90l179 178l467-466z"
				/></svg
			>
		</button>
	</div>

	<hr class="border-gray-700 w-full" />

	<div class="flex flex-col items-center w-full">
		<button
			aria-label="Add user"
			title="Add user"
			onclick={toggleUserInput}
			class="relative flex flex-col items-center cursor-pointer hover:bg-neutral-600 w-full py-2"
		>
			<svg xmlns="http://www.w3.org/2000/svg" width="1.5rem" height="1.5rem" viewBox="0 0 2048 2048"
				><!-- Icon from Fluent UI MDL2 by Microsoft Corporation - https://github.com/microsoft/fluentui/blob/master/packages/react-icons-mdl2/LICENSE --><path
					fill="currentColor"
					d="M1024 0q141 0 272 36t244 104t207 160t161 207t103 245t37 272q0 141-36 272t-104 244t-160 207t-207 161t-245 103t-272 37q-141 0-272-36t-245-103t-207-160t-160-208t-103-244t-37-273q0-141 36-272t104-244t160-207t207-161T752 37t272-37m0 1920q124 0 238-32t214-90t181-140t140-181t91-214t32-239t-32-238t-90-214t-140-181t-181-140t-214-91t-239-32t-238 32t-214 90t-181 140t-140 181t-91 214t-32 239t32 238t90 214t140 182t181 140t214 90t239 32m64-961h448v128h-448v449H960v-449H512V959h448V512h128z"
				/></svg
			>
			{#if showInput}
				<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
				<form onsubmit={async () => await addUser(username)} onkeydown={handleKeyDown}>
					<input
						bind:this={inputEl}
						bind:value={username}
						type="text"
						placeholder="Channel name"
						spellcheck="false"
						autocomplete="on"
						class="absolute top-2 left-12 px-2 shadow-md w-32 z-50 bg-gray-800 border border-white rounded-md outline-none user-select outline"
					/>
				</form>
			{/if}
		</button>

		{#if currentView.id !== 'users'}
			<button
				aria-label="Refresh"
				title="Refresh"
				disabled={loading}
				onclick={async () => await refreshFeed()}
				class="flex flex-col items-center cursor-pointer w-full py-2 {loading
					? 'opacity-50'
					: 'hover:bg-neutral-600'}"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="1.5rem"
					height="1.5rem"
					viewBox="0 0 2048 2048"
					><!-- Icon from Fluent UI MDL2 by Microsoft Corporation - https://github.com/microsoft/fluentui/blob/master/packages/react-icons-mdl2/LICENSE --><path
						fill="currentColor"
						d="M1297 38q166 45 304 140t237 226t155 289t55 331q0 141-36 272t-103 245t-160 207t-208 160t-245 103t-272 37q-141 0-272-36t-245-103t-207-160t-160-208t-103-244t-37-273q0-140 37-272t105-248t167-212t221-164H256V0h512v512H640V215q-117 56-211 140T267 545T164 773t-36 251q0 123 32 237t90 214t141 182t181 140t214 91t238 32q123 0 237-32t214-90t182-141t140-181t91-214t32-238q0-150-48-289t-136-253t-207-197t-266-124z"
					/></svg
				>
			</button>
		{/if}
	</div>
</aside>
