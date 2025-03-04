// eslint-disable-next-line prefer-const
export let emotesMap: Record<string, Record<string, Emote>> = $state({});
// eslint-disable-next-line prefer-const
export let regexMap: Record<string, RegExp> = $state({});

type Emote = {
	name: string;
	url: string;
	width: number;
	height: number;
};

const escapeEmoteReg = new RegExp('[.*+?^${}()|[\\]\\\\]', 'g');

export async function addUserEmotes(username: string, newEmotes: Emote[]) {
	if (!newEmotes) return;
	if (emotesMap[username]) return;

	emotesMap[username] = {};

	for (let i = 0; i < newEmotes.length; i++) {
		const emote = newEmotes[i];
		emotesMap[username][emote.name] = emote;
	}

	const emoteNames = newEmotes.map((emote) => emote.name);

	if (emoteNames.length === 0) {
		regexMap[username] = new RegExp('');
		return;
	}

	const escapedEmoteNames = emoteNames.map((name) => name.replace(escapeEmoteReg, '\\$&'));
	regexMap[username] = new RegExp(`\\b(${escapedEmoteNames.join('|')})\\b`, 'g');
}
