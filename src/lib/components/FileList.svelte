<script>
	import {
		app,
		enter,
		isLiked,
		toggleFavorite,
		startRename,
		requestDelete,
		yank,
		cut,
		paste,
		startNewFolder,
		openEditor
	} from '$lib/state.svelte.js';
	import Icon from './Icon.svelte';
	import ContextMenu from './ContextMenu.svelte';

	let scroller = $state(null);
	let menu = $state(null);

	function openMenu(e, entry, i) {
		e.preventDefault();
		app.selected = i;

		const folders = app.browseMode === 'folders';
		const browser = app.view === 'browser';
		const items = [];

		if (entry.is_dir) {
			items.push({ label: 'Open', icon: 'folder', action: enter });
		} else {
			items.push({ label: 'Play', icon: 'play', action: enter });
			items.push({ label: 'Edit metadata', icon: 'music', action: openEditor });
			items.push({
				label: isLiked(entry.path) ? 'Unfavorite' : 'Favorite',
				icon: 'heart',
				action: toggleFavorite
			});
		}

		if (folders && browser) {
			items.push({ separator: true });
			items.push({ label: 'Copy', action: yank });
			items.push({ label: 'Cut', action: cut });
			if (app.clip) items.push({ label: 'Paste', action: paste });
			items.push({ label: 'New folder', icon: 'folder', action: startNewFolder });
			items.push({ label: 'Rename', action: startRename });
			items.push({ separator: true });
			items.push({ label: 'Delete', action: requestDelete, danger: true });
		}

		menu = { x: e.clientX, y: e.clientY, items };
	}

	function fmt(s) {
		const total = Math.floor(s || 0);
		const m = Math.floor(total / 60);
		const sec = total % 60;
		return `${m}:${sec.toString().padStart(2, '0')}`;
	}

	function rowClass(entry, i) {
		const selected = i === app.selected;
		const playing = !selected && app.current && entry.path === app.current.path;
		const cut = app.clip && app.clip.path === entry.path && app.clip.op === 'cut';
		return [selected && 'selected', playing && 'playing', cut && 'cut'].filter(Boolean).join(' ');
	}

	$effect(() => {
		const i = app.selected;
		if (!scroller) return;
		const el = scroller.children[i];
		if (el) el.scrollIntoView({ behavior: 'instant', block: 'nearest' });
	});
</script>

{#if app.entries.length === 0}
	<div class="empty">{app.view === 'liked' ? 'No music' : 'Empty'}</div>
{:else}
	<div class="list" bind:this={scroller}>
		{#each app.entries as entry, i (entry.path)}
			<div
				class="row {rowClass(entry, i)}"
				role="button"
				tabindex="-1"
				onclick={() => (app.selected = i)}
				oncontextmenu={(e) => openMenu(e, entry, i)}
				ondblclick={() => {
					app.selected = i;
					enter();
				}}
				onkeydown={(e) => {
					if (e.key === 'Enter') {
						app.selected = i;
						enter();
					}
				}}
			>
				<span class="icon"><Icon name={entry.is_dir ? 'folder' : 'music'} size={15} /></span>
				{#if !entry.is_dir && isLiked(entry.path)}
					<span class="heart"><Icon name="heart" size={12} /></span>
				{/if}
				<span class="name">{entry.is_dir ? entry.name : entry.title || entry.name}</span>
				<span class="spacer"></span>
				{#if !entry.is_dir && entry.duration > 0}
					<span class="dur">{fmt(entry.duration)}</span>
				{/if}
			</div>
		{/each}
	</div>
{/if}

{#if menu}
	<ContextMenu x={menu.x} y={menu.y} items={menu.items} onclose={() => (menu = null)} />
{/if}

<style>
	.list {
		flex: 1;
		overflow-y: auto;
		min-height: 0;
	}

	.empty {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-muted);
		font-size: 16px;
	}

	.row {
		height: 25px;
		display: flex;
		align-items: center;
		padding: 0 6px;
		font-family: 'Undefined', monospace;
		cursor: pointer;
		color: var(--text);
		border-radius: 0;
	}

	.row:hover:not(.selected) {
		background: var(--hover);
	}

	.icon {
		flex: 0 0 auto;
		font-size: 14px;
		margin-right: 6px;
	}

	.heart {
		flex: 0 0 auto;
		font-size: 11px;
		color: var(--accent);
		margin-right: 4px;
	}

	.name {
		font-size: 18px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.spacer {
		flex: 1 1 auto;
	}

	.dur {
		flex: 0 0 auto;
		font-size: 14px;
		color: var(--text-faint);
		text-align: right;
		margin-left: 8px;
	}

	.row.cut {
		color: var(--text-muted);
	}

	.row.selected {
		background: var(--accent-soft);
		box-shadow: inset 0 0 0 1px var(--accent);
		color: var(--accent);
	}

	.row.selected .icon,
	.row.selected .name {
		color: var(--accent);
	}

	.row.playing {
		box-shadow: inset 0 0 0 2px var(--accent);
		color: var(--text);
	}
</style>
