import { fetch } from '@tauri-apps/plugin-http';
import { load, Store } from '@tauri-apps/plugin-store';

// eslint-disable-next-line prefer-const
export let usersMap: Record<string, User> = $state({});

export type User = {
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
	const response = await fetch('http://127.0.0.1:3030/live?usernames=' + usernames.join(','));

	if (response.status !== 200) {
		const parsed = await response.json();
		console.log('Error fetching', response.statusText, parsed.message);
		return;
	}

	const data = await response.json();

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
