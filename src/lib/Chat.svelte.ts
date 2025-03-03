import { currentStream } from './Stream.svelte';

let socket: WebSocket;

export function initChat(newMessageHandler: Function) {
	socket = new WebSocket('wss://irc-ws.chat.twitch.tv:443');

	socket.addEventListener('open', function () {
		socket.send('PASS SCHMOOPIIE');
		socket.send('NICK justinfan12345');
	});

	socket.addEventListener('message', function (event) {
		if (event.data.startsWith('PING')) {
			return;
		}

		const message = parseIRC(event.data);
		if (!message || !message.u || !message.m) return;

		newMessageHandler(message);
	});
}

export function joinChat(newChatChannel: string) {
	if (currentStream.username && currentStream.username !== newChatChannel) {
		socket.send('PART #' + currentStream.username);
	}

	socket.send('JOIN #' + newChatChannel);
}

const regex = new RegExp('^:(\\S+)!.+ PRIVMSG .+? :(.+?)$', 'm');

function parseIRC(message: string) {
	const match = message.match(regex);

	if (!match) return null;

	return {
		u: match[1],
		m: match[2]
	};
}
