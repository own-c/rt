import { goto } from '$app/navigation';

// eslint-disable-next-line prefer-const
export let currentView = $state({ id: 'streams', name: 'Streams' });

export function changeView(newViewID: string, gotoURL = true, path?: string) {
	switch (newViewID) {
		case 'videos':
			localStorage.setItem('lastView', newViewID);

			currentView.id = 'videos';
			currentView.name = 'Videos';
			if (gotoURL) {
				goto(`/videos${path ? `${path}` : ''}`);
			}
			break;

		case 'streams':
			localStorage.setItem('lastView', newViewID);

			currentView.id = 'streams';
			currentView.name = 'Streams';
			if (gotoURL) {
				goto(`/streams${path ? `${path}` : ''}`);
			}
			break;

		case 'users':
			localStorage.setItem('lastView', newViewID);

			currentView.id = 'users';
			currentView.name = 'Users';
			if (gotoURL) {
				goto('/users');
			}
			break;
	}
}
