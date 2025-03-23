import { goto } from '$app/navigation';

// eslint-disable-next-line prefer-const
export let currentView = $state({ id: 'twitch', name: 'Twitch' });

export function changeView(newView: string) {
	switch (newView) {
		case 'twitch':
			currentView.id = 'twitch';
			currentView.name = 'Twitch';
			goto('/view/twitch');
			break;

		case 'youtube':
			currentView.id = 'youtube';
			currentView.name = 'Youtube';
			goto('/view/youtube');
			break;
	}
}
