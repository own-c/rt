import type { ChatMessage } from './Types';

export type ChatEvent = {
	event: 'message';
	data: ChatMessage;
};
