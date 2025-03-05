<script lang="ts">
	import { onMount } from 'svelte';

	import { openUrl } from '@tauri-apps/plugin-opener';

	import { closeExistingChat, joinChat, URLReg, type ChatMessage } from '$lib/logic/Chat.svelte';
	import { emotesMap, regexMap } from '$lib/logic/Emotes.svelte';

	let { username, isLive } = $props();

	let messages: ChatMessage[] = $state([]);
	let tempMessages: ChatMessage[] = $state([]);

	let chatContainer: HTMLDivElement = $state(document.createElement('div'));
	let autoScroll = $state(true);

	function handleScroll() {
		if (chatContainer) {
			const { scrollTop, scrollHeight, clientHeight } = chatContainer;
			if (scrollTop + clientHeight < scrollHeight - 10) {
				autoScroll = false;
				return;
			}

			autoScroll = true;
			if (tempMessages.length > 0) {
				let combined = [...messages, ...tempMessages];
				if (combined.length > 500) {
					combined = combined.slice(combined.length - 500);
				}

				messages = combined;
				tempMessages = [];
			}
		}
	}

	let URLAndEmotesReg = $state(new RegExp(''));

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
		if (autoScroll && chatContainer && messages.length > 0) {
			chatContainer.scrollTop = chatContainer.scrollHeight;
		}
	});

	onMount(() => {
		if (!username || !isLive) return;

		URLAndEmotesReg = new RegExp(
			`(${URLReg.source})|(${regexMap[username].source})|([\\s]+)|([^\\s]+)`,
			'gm'
		);

		messages = [];
		tempMessages = [];

		joinChat(username, function (message: ChatMessage) {
			if (!autoScroll) {
				tempMessages = [...tempMessages, message];
			} else {
				if (messages.length < 500) {
					messages = [...messages, message];
				} else {
					messages = [...messages.slice(1), message];
				}
			}
		});

		return () => {
			closeExistingChat();
		};
	});
</script>

<div class="relative h-[calc(100vh-2rem)]" style="user-select: text;">
	<div
		bind:this={chatContainer}
		onscroll={handleScroll}
		class="bg-neutral-900 overflow-y-auto border-l-2 border-white/20 chat-container overflow-x-hidden h-full h-[calc(100vh-2rem)] max-h-[calc(100vh-2rem)]"
	>
		{#each messages as message, index (index)}
			<div class="text-pretty hover:bg-neutral-600 w-full px-2">
				<span class="font-bold" style="color: {message.c}"
					>{message.n}<span class="text-white">:</span></span
				>

				{#each message.m.match(URLAndEmotesReg) || [] as fragment, index (index)}
					{#if fragment.match(URLReg)}
						<button
							id="url"
							onclick={openUrlInBrowser}
							role="link"
							class="text-blue-400 hover:text-blue-600 underline underline-blue-400 cursor-pointer text-wrap break-words"
						>
							{fragment}
						</button>
					{:else if fragment.match(regexMap[username])}
						{@const emote = emotesMap[username][fragment]}
						<img
							class="inline-block align-middle"
							src={emote.u}
							alt={fragment}
							width={emote.w}
							height={emote.w}
							title={fragment}
						/>
					{:else}
						<span class="text-white">{fragment}</span>
					{/if}
				{/each}
			</div>
		{/each}
	</div>

	{#if !autoScroll}
		<button
			class="absolute bottom-0 left-1/2 transform -translate-x-1/2 text-white text-center p-1 bg-slate-800/80 hover:bg-slate-600/90 rounded-md shadow-lg z-50 cursor-pointer"
			onclick={() => {
				chatContainer.scrollTop = chatContainer.scrollHeight;
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

<style>
	.chat-container::-webkit-scrollbar {
		width: 10px;
	}

	.chat-container::-webkit-scrollbar-track {
		background: transparent;
	}

	.chat-container::-webkit-scrollbar-thumb {
		background-color: rgba(255, 255, 255, 0.4);
		border-radius: 4px;
	}
</style>
