import { fetch } from '@tauri-apps/plugin-http';

import { addUserEmotes } from './Emotes.svelte';
import { joinChat } from './Chat.svelte';
import { setUser, allUsers } from './Users.svelte';

export let currentStream = $state({
	username: '',
	title: '',
	live: false,
	url: ''
});

export async function switchStream(username) {
	if (!username) {
		console.log('No username');
		return;
	}

	const response = await fetch('http://127.0.0.1:3030/user/' + username);

	if (response.status !== 200) {
		const parsed = await response.json();
		console.log('Error fetching', response.statusText, parsed);

		if (!allUsers[username]) {
			let newUser = {
				username: username,
				live: false
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

	let newUser = {
		username: username,
		avatar: data.avatar,
		live: true
	};

	await setUser(newUser);
}
