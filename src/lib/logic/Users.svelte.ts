import { invoke } from '@tauri-apps/api/core';
import { load, type Store } from '@tauri-apps/plugin-store';

// eslint-disable-next-line prefer-const
export let usersMap: Record<string, User> = $state({});

type User = {
	username: string;
	avatar: string;
	live: boolean;
};

let tauriStore: Store;

export async function initUsers() {
	tauriStore = await load('users.json', { autoSave: false });
	const data = await tauriStore.get<Record<string, User>>('users');

	if (!data) return;

	for (const [key, value] of Object.entries(data)) {
		usersMap[key] = value;
	}
}

async function saveUsers() {
	await tauriStore.set('users', usersMap);
	await tauriStore.save();
}

export async function setUser(newUser: User) {
	usersMap[newUser.username] = newUser;
	await saveUsers();
}

export async function refreshUsers() {
	const usernames = Object.keys(usersMap);
	const data: string[] = await invoke('get_live_now', { usernames: usernames });

	Object.values(usersMap).forEach((user) => {
		if (data.includes(user.username)) {
			user.live = true;
		} else {
			user.live = false;
		}
	});

	await saveUsers();
}

export async function removeUser(username: string) {
	if (!usersMap[username]) return;

	delete usersMap[username];
	await saveUsers();
}
