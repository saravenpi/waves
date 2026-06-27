import { invoke } from '@tauri-apps/api/core';

export async function loadConfig() {
	return invoke('get_config');
}

export async function saveConfig(config) {
	return invoke('set_config', { config });
}

export function hexToRgb(hex) {
	const h = hex.replace('#', '');
	const n = parseInt(h.length === 3 ? h.replace(/(.)/g, '$1$1') : h, 16);
	return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

export function applyAccent(hex) {
	const [r, g, b] = hexToRgb(hex);
	const root = document.documentElement.style;
	root.setProperty('--accent', hex);
	root.setProperty('--accent-rgb', `${r}, ${g}, ${b}`);
	root.setProperty('--accent-soft', `rgba(${r}, ${g}, ${b}, 0.16)`);
	root.setProperty('--accent-faint', `rgba(${r}, ${g}, ${b}, 0.08)`);
}

export const COLOR_PRESETS = [
	'#9664FF',
	'#4A90E2',
	'#50E3C2',
	'#FF6B9D',
	'#FF8A00',
	'#FF4444',
	'#3F9D79'
];

export const ANIMATION_TYPES = ['spectrum', 'circle_spectrum', 'agbe', 'dots'];
