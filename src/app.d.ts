declare global {
	type User = {
		id: string;
		username: string;
		avatar: string;
		stream: Stream | null;
	};

	type Stream = {
		title: string;
		started_at: string;
		game: string;
		boxart: string;
		view_count: number;
		url: string | null;
	};

	type Watching = {
		username: string;
		title: string;
		started_at: string;
		game: string;
		boxart: string;
		view_count: number;
		last_update: Date;
		url: string | null;
	};

	type LiveNow = {
		username: string;
		started_at: string;
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
