import { fetch } from '@tauri-apps/plugin-http';

import { addUserEmotes } from './Emotes.svelte';
import { joinChat } from './Chat.svelte';
import { setUser, usersMap, type User } from './Users.svelte';

type Stream = {
	username: string;
	title: string;
	live: boolean;
	url: string;
};

export let currentStream: Stream = $state({
	username: '',
	title: '',
	live: false,
	url: ''
});

export async function switchStream(username: string) {
	if (!username) {
		console.log('No username');
		return;
	}

	const response = await fetch('http://127.0.0.1:3030/user/' + username);

	if (response.status !== 200) {
		const parsed = await response.json();
		console.log('Error fetching', response.statusText, parsed);

		if (!usersMap[username]) {
			const newUser: User = {
				username: username,
				live: false,
				avatar: ''
			};

			await setUser(newUser);
		}

		return;
	}

	const data = await response.json();

	await addUserEmotes(username, data.emotes);

	joinChat(data.username);
	currentStream.username = data.username;
	currentStream.title = data.title;
	currentStream.url = data.url;
	currentStream.live = data.live;

	let newUser = {
		username: username,
		live: data.live,
		avatar: data.avatar
	};

	await setUser(newUser);
}
