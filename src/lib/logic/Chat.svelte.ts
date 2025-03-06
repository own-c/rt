export type ChatMessage = {
	// Color
	c: string;
	// First message, not used
	f: string;
	// Name
	n: string;
	// Fragments that make up the message
	m: MessageFragment[];
};

type MessageFragment = {
	// Type, 0 = text, 1 = emote, 2 = url
	t: number;
	// Content
	c: string;
	// Emote
	e: Emote;
};

type Emote = {
	n: string;
	u: string;
	w: number;
	h: number;
};

export const URLReg = new RegExp(
	'(https?:\\/\\/)?(www\\.)?([a-zA-Z0-9-]{1,256})\\.[a-zA-Z0-9]{2,}(\\/[^\\s]*)?',
	'gm'
);

let sse: EventSource;

export function joinChat(username: string, newMessageHandler: (message: ChatMessage) => void) {
	closeExistingChat();

	sse = new EventSource(`http://127.0.0.1:3030/chat/${username}`);

	sse.onmessage = function (event: MessageEvent) {
		const data = JSON.parse(event.data);
		newMessageHandler(data);
	};
}

export function closeExistingChat() {
	if (sse) {
		sse.close();
	}
}
