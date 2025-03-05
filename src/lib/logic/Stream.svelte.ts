import { fetch } from '@tauri-apps/plugin-http';

import { setUser } from './Users.svelte';
import { fetchUserEmotes, setUserEmotes } from './Emotes.svelte';

type Watching = {
	username: string;
	url: string;
	live: boolean;
};

type StreamResponse = {
	live: boolean;
	avatar: string;
	url: string;
	username: string;
};

// eslint-disable-next-line prefer-const
export let watching: Watching = $state({
	username: '',
	url: '',
	live: false
});

export async function fetchStream(username: string) {
	const response = await fetch(`http://127.0.0.1:3030/stream/${username}`);

	if (response.status !== 200) {
		return;
	}

	const data: StreamResponse = await response.json();
	return data;
}

export function setWatching(stream: StreamResponse) {
	watching.username = stream.username;
	watching.url = stream.url;
	watching.live = stream.live;
}

export async function fetchAndSetStream(username: string) {
	const stream = await fetchStream(username);
	if (stream && stream.live) {
		const newUser = {
			username: username,
			live: stream.live,
			avatar: stream.avatar
		};

		await setUser(newUser);

		const emotes = await fetchUserEmotes(username);
		if (emotes) {
			setUserEmotes(username, emotes);
		}

		setWatching(stream);
		return;
	}

	setWatching({
		username: username,
		url: '',
		live: false,
		avatar: ''
	});
}
