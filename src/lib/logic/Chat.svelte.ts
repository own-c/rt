export type ChatMessage = {
	// Timestamp, not used
	t: string;
	// Color
	c: string;
	// First message, not used
	f: string;
	// Name
	n: string;
	// Message
	m: string;
};

const chatReg = new RegExp(
	'\\$TIMESTAMP:\\s*(\\d+)\\s+\\$COLOR:\\s*(#[A-Fa-f0-9]{6})?\\s+\\$FIRST_MSG:\\s*(\\d+)\\s+\\$NAME:\\s*(\\S+)\\s+\\$MESSAGE:\\s*(.+)',
	'gm'
);

let sse: EventSource;

export function joinChat(username: string, newMessageHandler: (message: ChatMessage) => void) {
	if (sse) {
		sse.close();
	}

	sse = new EventSource(`http://127.0.0.1:3030/chat/${username}`);

	sse.onmessage = function (event: MessageEvent) {
		const matches = Array.from((event.data as string).matchAll(chatReg));

		if (!matches) return;

		matches.forEach((match) => {
			if (match.length !== 6) return;

			const message = {
				t: match[1],
				c: match[2],
				f: match[3],
				n: match[4],
				m: match[5]
			};

			newMessageHandler(message);
		});
	};
}
