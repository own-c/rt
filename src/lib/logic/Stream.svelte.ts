import { invoke } from '@tauri-apps/api/core';

import { setUser } from './Users.svelte';

type Watching = {
	username: string;
	url: string;
	live: boolean;
};

type UserResponse = {
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

export async function fetchAndSetUser(username: string) {
	const stream: UserResponse = await invoke('get_user', { username: username });

	if (stream && stream.live) {
		const newUser = {
			username: username,
			live: stream.live,
			avatar: stream.avatar
		};

		await setUser(newUser);

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

function setWatching(stream: UserResponse) {
	watching.username = stream.username;
	watching.url = stream.url;
	watching.live = stream.live;
}
