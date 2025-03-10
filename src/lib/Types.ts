export type ChatMessage = {
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
	n: string;
	u: string;
	w: number;
	h: number;
};

export type StreamInfo = {
	title: string;
	started_at: string;
	game: string;
	view_count: number;
	box_art: string;
};
