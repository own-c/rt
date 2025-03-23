export function getAvatarUrl(avatar: number[]) {
	const byteArray = new Uint8Array(avatar);
	const blob = new Blob([byteArray], { type: 'image/png' });
	return URL.createObjectURL(blob);
}

export function streamingFor(startedAt: string) {
	const diff = new Date().getTime() - new Date(startedAt).getTime();
	const totalSeconds = Math.floor(diff / 1000);
	const hours = Math.floor(totalSeconds / 3600);
	const minutes = Math.floor((totalSeconds % 3600) / 60);
	const seconds = totalSeconds % 60;

	const formattedMinutes = minutes.toString().padStart(2, '0');
	const formattedSeconds = seconds.toString().padStart(2, '0');

	return `${hours}:${formattedMinutes}:${formattedSeconds}`;
}
