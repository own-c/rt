import { dev } from '$app/environment';
import { goto } from '$app/navigation';

// When in Release mode, the webview complains about the path not being found and then tries to load the index.html file.
// When in Debug mode, /index.html errors out.
const indexPath = dev ? '' : '/index.html';

// eslint-disable-next-line prefer-const
export let currentView = $state({ id: 'streams', name: 'Streams' });

export function changeView(newViewID: string, gotoURL = true, path?: string) {
	switch (newViewID) {
		case 'videos':
			localStorage.setItem('lastView', newViewID);

			currentView.id = 'videos';
			currentView.name = 'Videos';
			if (gotoURL) {
				goto(`/videos${path ? `${path}` : indexPath}`);
			}
			break;

		case 'streams':
			localStorage.setItem('lastView', newViewID);

			currentView.id = 'streams';
			currentView.name = 'Streams';
			if (gotoURL) {
				goto(`/streams${path ? `${path}` : indexPath}`);
			}
			break;

		case 'users':
			localStorage.setItem('lastView', newViewID);

			currentView.id = 'users';
			currentView.name = 'Users';
			if (gotoURL) {
				goto(`/users${indexPath}`);
			}
			break;
	}
}
