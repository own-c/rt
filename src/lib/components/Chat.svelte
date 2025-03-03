<script lang="ts">
	import { onMount, tick } from 'svelte';

	// Not sure if needed, but it doesn't hurt.
	import DOMPurify from 'dompurify';

	import { currentStream } from '$lib/Stream.svelte';
	import { regexMap, emotesMap } from '$lib/Emotes.svelte';
	import { initChat, type ChatMessage } from '$lib/Chat.svelte';

	const urlRegex = new RegExp(
		'https?://(www.)?[-a-zA-Z0-9@:%._+~#=]{1,256}\\.[a-zA-Z0-9()]{1,6}\\b([-a-zA-Z0-9()@:%_+.~#?&//=]*)'
	);

	let members: Record<string, string> = {};

	let colors = [
		'text-red-400',
		'text-orange-400',
		'text-yellow-400',
		'text-green-400',
		'text-teal-400',
		'text-blue-400',
		'text-purple-400',
		'text-gray-400',
		'text-indigo-400',
		'text-cyan-400',
		'text-emerald-400',
		'text-lime-400',
		'text-amber-400',
		'text-rose-400',
		'text-fuchsia-400',
		'text-sky-400',
		'text-pink-400'
	];

	function getMemberColor(member: string) {
		if (members[member]) {
			return members[member];
		}

		const color = colors[Math.floor(Math.random() * colors.length)];
		members[member] = color;
		return color;
	}

	function runRegex(text: string) {
		text = text.replace(urlRegex, (match) => {
			return `<a href="${match}" target="_blank" rel="noopener noreferrer" class="text-blue-400 hover:text-blue-600 underline underline-blue-400">${match}</a>`;
		});

		if (!regexMap[currentStream.username]) return DOMPurify.sanitize(text);

		text = text.replace(regexMap[currentStream.username], (match) => {
			const emote = emotesMap[currentStream.username][match];
			return emote
				? `<img class="inline-block align-middle" src="${emote.url}" alt="${match}" width="${emote.width}" height="${emote.height}" title="${match}">`
				: match;
		});

		return DOMPurify.sanitize(text);
	}

	let chatContainer: HTMLDivElement = $state(document.createElement('div'));
	let autoScroll = $state(true);

	let messages: ChatMessage[] = $state([]);
	let tempMessages: ChatMessage[] = $state([]);

	function handleScroll() {
		if (chatContainer) {
			const { scrollTop, scrollHeight, clientHeight } = chatContainer;
			if (scrollTop + clientHeight < scrollHeight - 10) {
				autoScroll = false;
			} else {
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
	}

	$effect(() => {
		if (autoScroll && chatContainer && messages.length > 0) {
			tick().then(() => {
				chatContainer.scrollTop = chatContainer.scrollHeight;
			});
		}
	});

	onMount(() => {
		initChat(function (message: ChatMessage) {
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
	});
</script>

<div class="relative" style="height: calc(100vh - 2.0rem);">
	<div
		bind:this={chatContainer}
		onscroll={handleScroll}
		class="bg-secondary-900 overflow-y-auto border-l-2 border-white/20 chat-container overflow-x-hidden h-full"
		style="height: calc(100vh - 2.0rem); max-height: calc(100vh - 2.0rem);"
	>
		{#each messages as message, index (index)}
			<!-- eslint-disable svelte/no-at-html-tags-->
			<div class="text-pretty hover:bg-secondary-600 w-full px-2 whitespace-pre-wrap break-words">
				<span class="font-bold {getMemberColor(message.u)}">{message.u}</span><!--
      --><span
					class="text-white">:&nbsp;</span
				><!--
      --><span class="text-white">{@html runRegex(message.m)}</span>
			</div>
			<!-- eslint-enable svelte/no-at-html-tags-->
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
