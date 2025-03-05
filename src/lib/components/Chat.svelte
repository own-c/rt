<script lang="ts" module>
	import { openUrl } from '@tauri-apps/plugin-opener';

	import { joinChat, type ChatMessage } from '$lib/logic/Chat.svelte';
	import { emotesMap, regexMap } from '$lib/logic/Emotes.svelte';
	import { watching } from '$lib/logic/Stream.svelte';

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

	const urlReg = new RegExp('(https?://[^s]+)', 'g');

	async function openUrlInBrowser(event: MouseEvent) {
		const target = event.target as HTMLElement;
		if (target.id === 'url') {
			await openUrl(target.innerText);
		}
	}

	export function setNewChat(username: string) {
		if (!username) return;

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

			if (autoScroll && chatContainer && messages.length > 0) {
				chatContainer.scrollTop = chatContainer.scrollHeight;
			}
		});
	}
</script>

<div class="relative" style="height: calc(100vh - 2.0rem);">
	<div
		bind:this={chatContainer}
		onscroll={handleScroll}
		class="bg-neutral-900 overflow-y-auto border-l-2 border-white/20 chat-container overflow-x-hidden h-full"
		style="height: calc(100vh - 2.0rem); max-height: calc(100vh - 2.0rem);"
	>
		{#each messages as message, index (index)}
			<div class="text-pretty hover:bg-neutral-600 w-full px-2 whitespace-pre-wrap break-words">
				<span class="font-bold" style="color: {message.c}"
					>{message.n}<span class="text-white">:&nbsp;</span></span
				>

				{#each message.m.split(urlReg) as part, index (index)}
					{#if part.match(urlReg)}
						<!-- svelte-ignore a11y_click_events_have_key_events -->
						<span
							id="url"
							onclick={openUrlInBrowser}
							role="link"
							tabindex={index}
							class="text-blue-400 hover:text-blue-600 underline underline-blue-400 cursor-pointer"
							>{part}</span
						>
					{:else if regexMap[watching.username] && emotesMap[watching.username]}
						{#if emotesMap[watching.username][part]}
							{@const emote = emotesMap[watching.username][part]}

							<img
								class="inline-block align-middle"
								src={emote.u}
								alt={part}
								width={emote.w}
								height={emote.w}
								title={part}
							/>
						{:else}
							<span class="text-white">{part}</span>
						{/if}
					{:else}
						<span class="text-white">{part}</span>
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
