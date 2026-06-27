import * as A from './state.svelte.js';
import { app } from './state.svelte.js';

let lastG = 0;

export function handleKey(e) {
	const k = e.key;

	if (app.prompt || app.editorOpen) {
		if (k === 'Escape') {
			e.preventDefault();
			if (app.prompt) A.cancelPrompt();
			else A.closeEditor();
		}
		return;
	}

	if (app.confirm) {
		if (k === 'Enter' || k === 'y') {
			e.preventDefault();
			A.confirmDelete();
		} else if (k === 'Escape' || k === 'n') {
			e.preventDefault();
			A.cancelConfirm();
		}
		return;
	}

	if (app.searchOpen) {
		if (k === 'ArrowDown') {
			e.preventDefault();
			A.searchMove(1);
		} else if (k === 'ArrowUp') {
			e.preventDefault();
			A.searchMove(-1);
		} else if (k === 'Enter') {
			e.preventDefault();
			A.searchEnter();
		} else if (k === 'Escape') {
			e.preventDefault();
			A.closeSearch();
		}
		return;
	}

	if (app.fullscreenViz && k === 'Escape') {
		e.preventDefault();
		app.fullscreenViz = false;
		return;
	}

	if (k === 'Tab') {
		e.preventDefault();
		A.cycleView(e.shiftKey ? -1 : 1);
		return;
	}
	if (k === '/') {
		e.preventDefault();
		A.openSearch();
		return;
	}
	if (k === ' ') {
		e.preventDefault();
		A.togglePlay();
		return;
	}
	if (k === 'ArrowLeft') {
		e.preventDefault();
		A.prev();
		return;
	}
	if (k === 'ArrowRight') {
		e.preventDefault();
		A.next();
		return;
	}
	if (k === 'ArrowUp') {
		e.preventDefault();
		A.changeVolume(0.05);
		return;
	}
	if (k === 'ArrowDown') {
		e.preventDefault();
		A.changeVolume(-0.05);
		return;
	}

	if (app.view === 'settings') {
		if (k === 'j') A.settingsMove(1);
		else if (k === 'k') A.settingsMove(-1);
		else if (k === 'h') A.settingsAdjust(-1);
		else if (k === 'l' || k === 'Enter') A.settingsActivate();
		return;
	}

	switch (k) {
		case 'h':
			A.back();
			break;
		case 'j':
			A.move(1);
			break;
		case 'k':
			A.move(-1);
			break;
		case 'l':
		case 'Enter':
			A.enter();
			break;
		case 'g': {
			const now = e.timeStamp;
			if (now - lastG < 500) A.jumpFirst();
			lastG = now;
			break;
		}
		case 'G':
			A.jumpLast();
			break;
		case 'b':
			A.cycleBrowseMode();
			break;
		case 'f':
			A.toggleFavorite();
			break;
		case 'm':
			A.openEditor();
			break;
		case 'n':
			A.startNewFolder();
			break;
		case 'r':
			A.startRename();
			break;
		case 'd':
			A.requestDelete();
			break;
		case 'y':
			A.yank();
			break;
		case 'x':
			A.cut();
			break;
		case 'p':
			A.paste();
			break;
		case 'Escape':
			A.clearClip();
			break;
	}
}
