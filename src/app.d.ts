declare global {
	type User = {
		id: string;
		username: string;
		platform: Platform;
		avatarBlob: number[];
	};

	enum Platform {
		Twitch = 'twitch',
		YouTube = 'youtube'
	}

	type Feed = {
		twitch: LiveNow[] | null;
	};

	type LiveNow = {
		username: string;
		started_at: string;
	};

	type ChatEvent = {
		event: 'message';
		data: ChatMessage;
	};

	type ChatMessage = {
		id: number;
		// Color
		c: string;
		// First message, not used
		f: boolean;
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
		// Name
		n: string;
		// URL
		u: string;
		// Width
		w: number;
		// Height
		h: number;
	};
}

export {};
