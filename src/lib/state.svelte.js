import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { engine } from './audio.js';
import { sounds } from './sounds.js';
import { Spectrum, BANDS } from './spectrum.js';
import {
	loadConfig,
	saveConfig,
	applyAccent,
	applyTransparency,
	COLOR_PRESETS,
	ANIMATION_TYPES
} from './config.js';

export const spectrumBars = new Float32Array(BANDS);
const spectrum = new Spectrum();

export const app = $state({
	ready: false,
	view: 'browser',
	browseMode: 'folders',
	root: '',
	cwd: '',
	stack: [],
	entries: [],
	selected: 0,
	library: [],
	groupLevel: 0,
	currentGroup: '',
	liked: [],
	likedPaths: new Set(),
	current: null,
	queue: [],
	queueIndex: -1,
	playing: false,
	currentTime: 0,
	duration: 0,
	volume: 0.8,
	loop: false,
	cover: null,
	waveform: [],
	searchOpen: false,
	searchQuery: '',
	searchResults: [],
	searchSelected: 0,
	editorOpen: false,
	editorTarget: null,
	clip: null,
	prompt: null,
	confirm: null,
	settingsSelected: 0,
	config: {
		animation: true,
		animation_type: 'spectrum',
		sidebar_position: 'left',
		sidebar_width: 500,
		decorations: true,
		window_corner_radius: 0,
		default_folder: null,
		show_status_bar: true,
		primary_color: '#9664FF',
		ui_sounds_enabled: true,
		ui_sounds_volume: 0.04,
		startup_sound_enabled: true,
		transparent: false
	}
});

let loadToken = 0;

function baseName(p) {
	return p.split('/').filter(Boolean).pop() || p;
}

export async function initApp() {
	const cfg = await loadConfig();
	app.config = cfg;
	applyAccent(cfg.primary_color);
	applyTransparency(cfg.transparent);
	sounds.configure(cfg.ui_sounds_enabled, cfg.ui_sounds_volume, cfg.startup_sound_enabled);

	app.liked = await invoke('get_liked');
	app.likedPaths = new Set(app.liked.map((l) => l.path));

	const dir = await invoke('default_music_dir', { defaultFolder: cfg.default_folder });
	app.root = dir;
	app.library = await invoke('scan_library', { root: dir });
	await openFolder(dir);

	startWatching();
	app.ready = true;
	sounds.startup();
}

export async function openFolder(path) {
	const entries = await invoke('list_dir', { path });
	app.cwd = path;
	app.entries = entries;
	app.selected = 0;
	app.groupLevel = 0;
	invoke('watch_dir', { path }).catch(() => {});
}

let watchTimer;
async function refreshCurrentDir() {
	if (app.view === 'browser' && app.browseMode === 'folders') {
		const prev = app.entries[app.selected]?.path;
		const entries = await invoke('list_dir', { path: app.cwd });
		app.entries = entries;
		const idx = entries.findIndex((e) => e.path === prev);
		app.selected = idx >= 0 ? idx : Math.min(app.selected, Math.max(0, entries.length - 1));
	} else {
		app.library = await invoke('scan_library', { root: app.root });
		rebuildEntries();
	}
}

export function startWatching() {
	listen('dir-changed', () => {
		clearTimeout(watchTimer);
		watchTimer = setTimeout(refreshCurrentDir, 250);
	});
}

function uniqueGroups(key) {
	const map = new Map();
	for (const t of app.library) {
		const g = (t[key] || 'Unknown').trim() || 'Unknown';
		if (!map.has(g)) map.set(g, 0);
		map.set(g, map.get(g) + 1);
	}
	return [...map.keys()].sort((a, b) => a.toLowerCase().localeCompare(b.toLowerCase()));
}

function groupTracks(key, value) {
	const v = value === 'Unknown' ? '' : value;
	return app.library.filter((t) => ((t[key] || '').trim() || 'Unknown') === (v ? value : 'Unknown'));
}

export function rebuildEntries() {
	if (app.view === 'liked') {
		app.entries = app.liked.map((l) => ({
			name: l.name,
			path: l.path,
			is_dir: false,
			title: l.name,
			artist: '',
			album: '',
			duration: 0
		}));
		app.selected = Math.min(app.selected, Math.max(0, app.entries.length - 1));
		return;
	}
	if (app.browseMode === 'folders') {
		return;
	}
	if (app.browseMode === 'allsongs') {
		app.entries = app.library.slice();
		app.selected = 0;
		return;
	}
	const key = app.browseMode === 'artists' ? 'artist' : 'album';
	if (app.groupLevel === 0) {
		app.entries = uniqueGroups(key).map((g) => ({
			name: g,
			path: `group:${g}`,
			is_dir: true,
			title: g,
			artist: '',
			album: '',
			duration: 0
		}));
	} else {
		app.entries = groupTracks(key, app.currentGroup);
	}
	app.selected = 0;
}

export function setView(v) {
	app.view = v;
	if (v === 'liked') rebuildEntries();
	else if (v === 'browser') rebuildEntries();
	sounds.cursor();
}

export function cycleView(dir) {
	const order = ['browser', 'liked', 'settings'];
	const i = order.indexOf(app.view);
	setView(order[(i + dir + order.length) % order.length]);
}

export function setBrowseMode(m) {
	app.browseMode = m;
	app.groupLevel = 0;
	if (m === 'folders') openFolder(app.root);
	else rebuildEntries();
	sounds.cursor();
}

export function cycleBrowseMode() {
	const order = ['folders', 'artists', 'albums', 'allsongs'];
	const i = order.indexOf(app.browseMode);
	setBrowseMode(order[(i + 1) % order.length]);
}

export function move(delta) {
	if (!app.entries.length) return;
	app.selected = (app.selected + delta + app.entries.length) % app.entries.length;
	sounds.cursor();
}

export function jumpFirst() {
	app.selected = 0;
	sounds.cursor();
}

export function jumpLast() {
	app.selected = Math.max(0, app.entries.length - 1);
	sounds.cursor();
}

function audioQueue() {
	return app.entries.filter((e) => !e.is_dir);
}

export function enter() {
	const e = app.entries[app.selected];
	if (!e) return;
	if (e.is_dir) {
		if (app.view === 'browser' && app.browseMode === 'folders') {
			app.stack.push(app.cwd);
			openFolder(e.path);
		} else if (app.browseMode === 'artists' || app.browseMode === 'albums') {
			app.currentGroup = e.name;
			app.groupLevel = 1;
			rebuildEntries();
		}
		sounds.cursor();
	} else {
		const q = audioQueue();
		const idx = q.findIndex((t) => t.path === e.path);
		playQueue(q, Math.max(0, idx));
	}
}

export function back() {
	if (app.view === 'browser' && app.browseMode === 'folders') {
		if (app.stack.length) {
			const prev = app.stack.pop();
			openFolder(prev);
		} else if (app.cwd !== '/' && app.cwd) {
			const parent = app.cwd.split('/').slice(0, -1).join('/') || '/';
			openFolder(parent);
		}
	} else if ((app.browseMode === 'artists' || app.browseMode === 'albums') && app.groupLevel === 1) {
		app.groupLevel = 0;
		rebuildEntries();
	}
	sounds.cursor();
}

export async function playQueue(queue, index) {
	if (!queue.length) return;
	app.queue = queue;
	app.queueIndex = index;
	const track = queue[index];
	app.current = track;
	app.cover = null;
	app.waveform = [];
	app.currentTime = 0;
	const token = ++loadToken;

	await engine.load(track.path);
	engine.setVolume(app.volume);
	engine.setLoop(app.loop);
	engine.audio.onended = () => {
		if (!app.loop) next();
	};
	await engine.play();
	app.playing = true;

	invoke('read_cover', { path: track.path }).then((c) => {
		if (token === loadToken) app.cover = c;
	});
	engine.computeWaveform(track.path).then((w) => {
		if (token === loadToken) app.waveform = w;
	});
}

export async function togglePlay() {
	if (!app.current) {
		const q = audioQueue();
		if (q.length) playQueue(q, 0);
		return;
	}
	if (app.playing) {
		engine.pause();
		app.playing = false;
	} else {
		await engine.play();
		app.playing = true;
	}
}

export function next() {
	if (!app.queue.length) return;
	playQueue(app.queue, (app.queueIndex + 1) % app.queue.length);
}

export function prev() {
	if (!app.queue.length) return;
	if (engine.currentTime > 3) {
		engine.seek(0);
		return;
	}
	playQueue(app.queue, (app.queueIndex - 1 + app.queue.length) % app.queue.length);
}

export function changeVolume(delta) {
	app.volume = Math.min(1, Math.max(0, app.volume + delta));
	engine.setVolume(app.volume);
	sounds.cursor();
}

export function setVolumeValue(v) {
	app.volume = Math.min(1, Math.max(0, v));
	engine.setVolume(app.volume);
}

export function seekTo(t) {
	engine.seek(t);
	app.currentTime = t;
}

export function toggleLoop() {
	app.loop = !app.loop;
	engine.setLoop(app.loop);
}

export async function toggleFavorite() {
	const e = app.entries[app.selected];
	if (!e || e.is_dir) return;
	app.liked = await invoke('toggle_liked', { path: e.path, name: e.name, isDir: false });
	app.likedPaths = new Set(app.liked.map((l) => l.path));
	if (app.view === 'liked') rebuildEntries();
	sounds.cursor();
}

export function isLiked(path) {
	return app.likedPaths.has(path);
}

export function openSearch() {
	app.searchOpen = true;
	app.searchQuery = '';
	app.searchResults = [];
	app.searchSelected = 0;
}

export function closeSearch() {
	app.searchOpen = false;
	app.searchQuery = '';
	app.searchResults = [];
}

export function setSearch(q) {
	app.searchQuery = q;
	const needle = q.toLowerCase();
	if (!needle) {
		app.searchResults = [];
		return;
	}
	app.searchResults = app.library
		.filter(
			(t) =>
				t.title.toLowerCase().includes(needle) ||
				t.artist.toLowerCase().includes(needle) ||
				t.album.toLowerCase().includes(needle)
		)
		.slice(0, 5);
	app.searchSelected = 0;
}

export function searchMove(d) {
	if (!app.searchResults.length) return;
	app.searchSelected =
		(app.searchSelected + d + app.searchResults.length) % app.searchResults.length;
}

export function searchEnter() {
	const t = app.searchResults[app.searchSelected];
	if (t) playQueue(app.searchResults, app.searchSelected);
	closeSearch();
}

export function openEditor() {
	const e = app.entries[app.selected];
	if (!e || e.is_dir) return;
	app.editorTarget = e;
	app.editorOpen = true;
}

export function closeEditor() {
	app.editorOpen = false;
	app.editorTarget = null;
}

export async function saveEditor(fields) {
	const t = app.editorTarget;
	if (!t) return;
	await invoke('write_metadata', {
		path: t.path,
		title: fields.title,
		artist: fields.artist,
		album: fields.album,
		date: fields.date || ''
	});
	closeEditor();
	app.library = await invoke('scan_library', { root: app.root });
	if (app.browseMode === 'folders' && app.view === 'browser') await openFolder(app.cwd);
	else rebuildEntries();
}

export async function applyConfig(patch) {
	app.config = { ...app.config, ...patch };
	if (patch.primary_color) applyAccent(patch.primary_color);
	if (patch.transparent !== undefined) applyTransparency(app.config.transparent);
	sounds.configure(
		app.config.ui_sounds_enabled,
		app.config.ui_sounds_volume,
		app.config.startup_sound_enabled
	);
	await saveConfig(app.config);
}

async function refresh() {
	app.library = await invoke('scan_library', { root: app.root });
	if (app.view === 'browser' && app.browseMode === 'folders') await openFolder(app.cwd);
	else rebuildEntries();
}

function selectedEntry() {
	return app.entries[app.selected];
}

export function startRename() {
	const e = selectedEntry();
	if (!e) return;
	app.prompt = { kind: 'rename', value: e.name, target: e };
}

export function startNewFolder() {
	if (app.browseMode !== 'folders' || app.view !== 'browser') return;
	app.prompt = { kind: 'newfolder', value: '', target: null };
}

export async function submitPrompt(value) {
	const p = app.prompt;
	app.prompt = null;
	if (!p || !value) return;
	if (p.kind === 'rename') {
		await invoke('rename_path', { path: p.target.path, newName: value });
	} else if (p.kind === 'newfolder') {
		await invoke('new_folder', { parent: app.cwd, name: value });
	}
	await refresh();
}

export function cancelPrompt() {
	app.prompt = null;
}

export function requestDelete() {
	const e = selectedEntry();
	if (!e) return;
	app.confirm = { message: `Move "${e.name}" to Trash?`, target: e };
}

export async function confirmDelete() {
	const c = app.confirm;
	app.confirm = null;
	if (!c) return;
	await invoke('delete_path', { path: c.target.path });
	sounds.delete();
	await refresh();
}

export function cancelConfirm() {
	app.confirm = null;
}

export function yank() {
	const e = selectedEntry();
	if (!e) return;
	app.clip = app.clip && app.clip.path === e.path && app.clip.op === 'copy' ? null : { path: e.path, name: e.name, op: 'copy' };
	sounds.cursor();
}

export function cut() {
	const e = selectedEntry();
	if (!e) return;
	app.clip = app.clip && app.clip.path === e.path && app.clip.op === 'cut' ? null : { path: e.path, name: e.name, op: 'cut' };
	sounds.cursor();
}

export async function paste() {
	if (!app.clip || app.browseMode !== 'folders') return;
	const e = selectedEntry();
	const dest = e && e.is_dir ? e.path : app.cwd;
	await invoke('paste_path', { src: app.clip.path, destDir: dest, cut: app.clip.op === 'cut' });
	app.clip = null;
	sounds.cursor();
	await refresh();
}

export function clearClip() {
	if (app.clip) {
		app.clip = null;
		sounds.cursor();
	}
}

export const SETTINGS = [
	{ key: 'primary_color', label: 'Primary Color', type: 'cycle', options: COLOR_PRESETS },
	{ key: 'show_status_bar', label: 'Status Bar', type: 'toggle' },
	{ key: 'decorations', label: 'Window Decorations', type: 'toggle' },
	{ key: 'animation', label: 'Animation', type: 'toggle' },
	{ key: 'animation_type', label: 'Animation Type', type: 'cycle', options: ANIMATION_TYPES },
	{ key: 'sidebar_position', label: 'Sidebar Position', type: 'cycle', options: ['left', 'right'] },
	{ key: 'ui_sounds_enabled', label: 'UI Sounds', type: 'toggle' },
	{ key: 'ui_sounds_volume', label: 'UI Sounds Volume', type: 'range' },
	{ key: 'startup_sound_enabled', label: 'Startup Sound', type: 'toggle' },
	{ key: 'transparent', label: 'Transparency (macOS)', type: 'toggle' }
];

export function settingsMove(d) {
	app.settingsSelected = (app.settingsSelected + d + SETTINGS.length) % SETTINGS.length;
	sounds.cursor();
}

export function settingsAdjust(dir) {
	const item = SETTINGS[app.settingsSelected];
	if (item.type === 'toggle') {
		applyConfig({ [item.key]: !app.config[item.key] });
	} else if (item.type === 'cycle') {
		const opts = item.options;
		const i = opts.indexOf(app.config[item.key]);
		const ni = (i + dir + opts.length) % opts.length;
		applyConfig({ [item.key]: opts[ni] });
	} else if (item.type === 'range') {
		const v = Math.min(1, Math.max(0, (app.config[item.key] || 0) + dir * 0.05));
		applyConfig({ [item.key]: Math.round(v * 100) / 100 });
	}
	sounds.cursor();
}

export function settingsActivate() {
	settingsAdjust(1);
}

export function tick() {
	if (app.playing) {
		app.currentTime = engine.currentTime;
		app.duration = engine.duration;
	}
	spectrum.update(engine.analyser, app.playing && app.config.animation);
	spectrumBars.set(spectrum.bars);
}
