import { currentStream } from './Stream.svelte';

export type ChatMessage = {
	// Username
	u: string;
	// Message
	m: string;
};

const IRCReg = new RegExp('^:(\\S+)!.+ PRIVMSG .+? :(.+?)$', 'm');

export const URLReg = new RegExp(
	'https?://(www.)?[-a-zA-Z0-9@:%._+~#=]{1,256}\\.[a-zA-Z0-9()]{1,6}\\b([-a-zA-Z0-9()@:%_+.~#?&//=]*)'
);

let socket: WebSocket;

export function initChat(newMessageHandler: (message: ChatMessage) => void) {
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

function parseIRC(message: string) {
	const match = message.match(IRCReg);

	if (!match) return null;

	return {
		u: match[1],
		m: match[2]
	};
}
