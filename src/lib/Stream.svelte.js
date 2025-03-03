export let stream = $state({
    username: "",
    title: "",
    url: "",
})

let socket;

export function initChat(newHandler) {
    socket = new WebSocket("wss://irc-ws.chat.twitch.tv:443");

    socket.addEventListener("open", function (event) {
        socket.send("PASS SCHMOOPIIE");
        socket.send("NICK justinfan12345");
    });

    socket.addEventListener("message", function (event) {
        if (event.data.startsWith("PING")) {
            return;
        }

        const message = parseIRC(event.data);
        if (!message || !message.u || !message.m) return;

        newHandler(message);
    });
}

export function setNewStream(newStream) {
    joinChat(newStream.username);
    stream.username = newStream.username;
    stream.title = newStream.title;
    stream.url = newStream.url;
}

export function joinChat(newChatChannel) {
    if (stream.username && stream.username !== newChatChannel) {
        socket.send("PART #" + stream.username);
    }

    socket.send("JOIN #" + newChatChannel);
}


function parseIRC(message) {
    const regex = /^:(\S+)!.+ PRIVMSG .+? :(.+?)$/m;
    const match = message.match(regex);

    if (match) {
        return {
            u: match[1],
            m: match[2],
        };
    }

    return null;
}

