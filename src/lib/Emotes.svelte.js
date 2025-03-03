export let emotesMap = $state({});
export let regexMap = $state({});

const escapeEmoteReg = new RegExp('[.*+?^${}()|[\\]\\\\]', 'g');

export async function addUserEmotes(username, newEmotes) {
	if (!newEmotes) return;
	if (emotesMap[username]) return;

	emotesMap[username] = {};

	for (let i = 0; i < newEmotes.length; i++) {
		const emote = newEmotes[i];
		emotesMap[username][emote.name] = emote;
	}

	const emoteNames = newEmotes.map((emote) => emote.name);

	if (emoteNames.length === 0) {
		regexMap[username] = null;
		return;
	}

	const escapedEmoteNames = emoteNames.map((name) => name.replace(escapeEmoteReg, '\\$&'));
	regexMap[username] = new RegExp(`\\b(${escapedEmoteNames.join('|')})\\b`, 'g');
}
