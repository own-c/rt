import { invoke } from '@tauri-apps/api/core';
import { load, type Store } from '@tauri-apps/plugin-store';

import { error } from './components/Notification.svelte';

// eslint-disable-next-line prefer-const
export let users: Record<string, User> = $state({});

// eslint-disable-next-line prefer-const
export let live_now: Record<string, LiveNow> = $state({});

// eslint-disable-next-line prefer-const
export let watching: Watching = $state({
	username: '',
	title: '',
	started_at: '',
	game: '',
	boxart: '',
	view_count: 0,
	last_update: new Date(),
	url: null
});

let userStore: Store;

export async function initStores() {
	userStore = await load('users.json', { autoSave: false });
	const data = await userStore.get<Record<string, User>>('users');

	if (!data) return;

	for (const [key, value] of Object.entries(data)) {
		users[key] = value;
	}
}

export async function refreshUsers() {
	const usernames = Object.keys(users);
	if (usernames.length === 0) return;

	await invoke<Record<string, LiveNow>>('fetch_live_now', { usernames: usernames })
		.then((data) => {
			for (const [key, value] of Object.entries(data)) {
				live_now[key] = value;
			}
		})
		.catch((err) => {
			error(`Error fetching live now`, err);
		});
}

export async function updateUser(username: string, switchStream: boolean) {
	await invoke<User>('fetch_full_user', { username: username })
		.then(async (user) => {
			if (user.stream && switchStream) {
				await invoke<string>('fetch_stream_playback', {
					username: username,
					backup: false
				})
					.then((url) => {
						user.stream!.url = url;
						updateWatching(username, user.stream!);
					})
					.catch((err) => {
						error(`Error fetching stream playback access token`, err);
					});
			}

			// Don't save the stream info
			user.stream = null;

			await setUser(user);
		})
		.catch((err) => {
			error(`Error fetching user`, err);
		});
}

export async function setUser(newUser: User) {
	users[newUser.username] = newUser;
	await saveUsers();
}

export async function removeUser(username: string) {
	if (!users[username]) return;

	delete users[username];
	await saveUsers();
}

async function saveUsers() {
	await userStore.set('users', users);
	await userStore.save();
}

export function updateWatching(username: string, stream: Stream) {
	// If there is a url, it means we are switching streams
	// This forces the player to rerender
	if (stream.url) {
		watching.username = username;
		watching.url = stream.url!;
	}

	watching.title = stream.title;
	watching.game = stream.game;
	watching.boxart = stream.boxart;
	watching.view_count = stream.view_count;

	const startedAt = JSON.parse(JSON.stringify(stream.started_at));
	const startedAtDate = new Date(startedAt);

	watching.last_update = new Date();

	const diff = watching.last_update.getTime() - startedAtDate.getTime();
	const totalSeconds = Math.floor(diff / 1000);

	watching.started_at = parseTime(totalSeconds);
}

export function parseTime(totalSeconds: number) {
	const hours = Math.floor(totalSeconds / 3600);
	const minutes = Math.floor((totalSeconds % 3600) / 60);
	const seconds = totalSeconds % 60;

	const formattedMinutes = minutes.toString().padStart(2, '0');
	const formattedSeconds = seconds.toString().padStart(2, '0');

	return `${hours}:${formattedMinutes}:${formattedSeconds}`;
}
