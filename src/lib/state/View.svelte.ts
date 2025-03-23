import { goto } from '$app/navigation';

// eslint-disable-next-line prefer-const
export let currentView = $state({ id: 'streams', name: 'Streams' });

export function changeView(newView: string) {
	switch (newView) {
		case 'videos':
			currentView.name = 'Videos';
			goto('/videos');
			break;

		case 'streams':
			currentView.name = 'Streams';
			goto('/streams');
			break;

		case 'users':
			currentView.name = 'Users';
			goto('/users');
			break;
	}
}
