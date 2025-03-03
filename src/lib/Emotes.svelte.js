export let emotes = $state({});
export let regex = $state({});

export async function addEmotes(username, newEmotes) {
    if (!newEmotes) return;
    if (emotes[username]) return;

    emotes[username] = {};

    for (let i = 0; i < newEmotes.length; i++) {
        const emote = newEmotes[i];
        emotes[username][emote.name] = emote;
    }

    const emoteNames = newEmotes.map((emote) => emote.name);

    if (emoteNames.length === 0) {
        regex[username] = null;
        return;
    }

    const escapedEmoteNames = emoteNames.map((str) => str.replace(/[-\/\\^$*+?.()|[\]{}]/g, '\\$&'));
    regex[username] = new RegExp(`\\b(${escapedEmoteNames.join("|")})\\b`, "g");
}
