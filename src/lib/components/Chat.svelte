<script lang="ts">
	import { onMount } from 'svelte';

	import { openUrl } from '@tauri-apps/plugin-opener';
	import { Channel, invoke } from '@tauri-apps/api/core';

	import SimpleBar from 'simplebar';

	import type { ChatMessage } from '$lib/Types';
	import type { ChatEvent } from '$lib/Events';

	let { username, isLive } = $props();

	let messages: ChatMessage[] = $state([]);
	let tempMessages: ChatMessage[] = $state([]);
	let pendingMessages: ChatMessage[] = [];
	let updateScheduled = false;

	let chatContainer = $state() as HTMLDivElement;
	let simpleBarInstance = $state() as HTMLElement;
	let autoScroll = $state(true);

	function handleScroll() {
		const { scrollTop, scrollHeight, clientHeight } = simpleBarInstance;

		if (scrollTop + clientHeight < scrollHeight - 10) {
			autoScroll = false;
			return;
		}

		autoScroll = true;
		if (tempMessages.length > 0) {
			let combined = [...messages, ...tempMessages];
			if (combined.length > 300) {
				combined = combined.slice(combined.length - 300);
			}

			messages = combined;
			tempMessages = [];
		}
	}

	async function openUrlInBrowser(event: MouseEvent) {
		const target = event.target as HTMLElement;

		if (target.id === 'url') {
			let url = target.innerText;
			if (!url.startsWith('http') && !url.startsWith('https')) {
				url = `https://${url}`;
			}

			await openUrl(url);
		}
	}

	$effect(() => {
		if (autoScroll && simpleBarInstance && messages.length > 0) {
			simpleBarInstance.scrollTop = simpleBarInstance.scrollHeight;
		}
	});

	onMount(() => {
		if (!username || !isLive) return;

		simpleBarInstance = new SimpleBar(chatContainer).getScrollElement()!;
		simpleBarInstance.addEventListener('scroll', handleScroll);

		messages = [];
		tempMessages = [];
		pendingMessages = [];
		updateScheduled = false;

		const onEvent = new Channel<ChatEvent>();

		let id = 0;

		onEvent.onmessage = ({ event, data }) => {
			if (event === 'message' && data) {
				data.id = id++;

				if (!autoScroll) {
					tempMessages = [...tempMessages, data];
				} else {
					pendingMessages.push(data);

					if (!updateScheduled) {
						updateScheduled = true;

						requestAnimationFrame(() => {
							let combined = [...messages, ...pendingMessages];
							if (combined.length > 300) {
								combined = combined.slice(combined.length - 300);
							}

							messages = combined;
							pendingMessages = [];
							updateScheduled = false;
						});
					}
				}
			}
		};

		(async () => await invoke('join_chat', { username, onEvent }))();

		return () => {
			(async () => await invoke('leave_chat', { username }))();
		};
	});
</script>

<div class="relative h-[calc(100vh-2rem)]" style="user-select: text;">
	<div
		data-simplebar
		bind:this={chatContainer}
		class="bg-neutral-900 overflow-y-auto border-l-2 border-white/20 overflow-x-hidden h-full h-[calc(100vh-2rem)] max-h-[calc(100vh-2rem)] min-w-full"
	>
		{#each messages as message (message.id)}
			<div class="text-pretty hover:bg-neutral-600 w-full mx-1">
				<span class="font-bold" style="color: {message.c}"
					>{message.n}<span class="text-white">:</span></span
				>

				{#each message.m as fragment, index (index)}
					{#if fragment.t === 0}
						<span class="text-white break-words">{fragment.c}</span>
					{:else if fragment.t === 1 && fragment.e}
						<img
							loading="lazy"
							class="mx-2 inline-block align-middle"
							src={fragment.e.u}
							alt={fragment.e.n}
							width={fragment.e.w}
							height={fragment.e.h}
							title={fragment.e.n}
						/>
					{:else}
						<button
							id="url"
							onclick={openUrlInBrowser}
							role="link"
							class="mx-2 break-all text-blue-400 hover:text-blue-600 underline underline-blue-400 cursor-pointer"
						>
							{fragment.c}
						</button>
					{/if}
				{/each}
			</div>
		{/each}
	</div>

	{#if !autoScroll}
		<button
			class="absolute bottom-0 left-1/2 transform -translate-x-1/2 text-white text-center p-1 bg-slate-800/80 hover:bg-slate-600/90 rounded-md shadow-lg z-50 cursor-pointer"
			onclick={() => {
				simpleBarInstance.scrollTop = simpleBarInstance.scrollHeight;
			}}
		>
			<span class="shadow-lg">
				{#if tempMessages.length === 0}
					Chat paused
				{:else}
					{tempMessages.length} new messages
				{/if}
			</span>
		</button>
	{/if}
</div>
