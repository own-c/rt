// eslint-disable-next-line prefer-const
export let emotesMap: Record<string, Record<string, Emote>> = $state({});
// eslint-disable-next-line prefer-const
export let regexMap: Record<string, RegExp> = $state({});

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

const emoteReg = new RegExp('[.*+?^${}()|[\\]\\\\]', 'g');

export async function fetchUserEmotes(username: string) {
	const response = await fetch(`http://127.0.0.1:3030/emotes/${username}`);

	if (response.status !== 200) {
		return;
	}

	const data: Emote[] = await response.json();
	return data;
}

export function setUserEmotes(username: string, newEmotes: Emote[]) {
	if (!newEmotes) return;
	if (emotesMap[username]) return;

	emotesMap[username] = {};

	for (let i = 0; i < newEmotes.length; i++) {
		const emote = newEmotes[i];
		emotesMap[username][emote.n] = emote;
	}

	const names = newEmotes.map((emote) => emote.n);

	if (names.length === 0) {
		regexMap[username] = new RegExp('');
		return;
	}

	const escaped = names.map((name) => name.replace(emoteReg, '\\$&'));
	regexMap[username] = new RegExp(`\\b(${escaped.join('|')})\\b`, 'g');
}
