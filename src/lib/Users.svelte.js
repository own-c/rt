import { fetch } from "@tauri-apps/plugin-http";
import { load } from "@tauri-apps/plugin-store";

export let users = $state({})

let tauriStore;

export async function loadUsers() {
    tauriStore = await load("users.json", { autoSave: false });
    const data = await tauriStore.get("users");
    if (!data) return;

    for (const [key, value] of Object.entries(data)) {
        users[key] = value;
    }
}

export async function saveUsers() {
    await tauriStore.set("users", users);
    await tauriStore.save();
}

export async function setUser(newUser) {
    users[newUser.username] = newUser;
    await saveUsers();
}

export async function refreshUsers() {
    const usernames = Object.keys(users);
    const response = await fetch(
        "http://127.0.0.1:3030/live?usernames=" + usernames.join(","),
    );

    if (response.status !== 200) {
        const parsed = await response.json();
        console.log("Error fetching", response.statusText, parsed.message);
        return;
    }

    const data = await response.json();

    Object.values(users).forEach((user) => {
        if (data.includes(user.username)) {
            user.live = true;
        } else {
            user.live = false;
        }
    });

    await saveUsers();
}

export async function removeUser(username) {
    if (!users[username]) return;

    delete users[username];
    await saveUsers();
}

