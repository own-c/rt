// eslint-disable-next-line prefer-const
export let emotesMap: Record<string, Record<string, Emote>> = $state({});
// eslint-disable-next-line prefer-const
export let regexMap: Record<string, RegExp> = $state({});

export type Emote = {
	// Name
	n: string;
	// URL
	u: string;
	// Width
	w: number;
	// Height
	h: number;
};

const emoteReg = new RegExp('[.*+?^${}()|[\\]\\\\]', 'g');

export function setUserEmotes(username: string, newEmotes: Record<string, Emote>) {
	if (!newEmotes) return;
	if (emotesMap[username]) return;

	emotesMap[username] = {};

	Object.entries(newEmotes).forEach(([emoteName, emote]) => {
		emotesMap[username][emoteName] = emote;
	});

	const names = Object.keys(newEmotes);

	if (names.length === 0) {
		regexMap[username] = new RegExp('');
		return;
	}

	const escaped = names.map((name) => name.replace(emoteReg, '\\$&'));
	regexMap[username] = new RegExp(`\\b(${escaped.join('|')})\\b`, 'g');
}
