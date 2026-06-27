<script>
	import { app, setView, setBrowseMode, back } from '$lib/state.svelte.js';
	import FileList from './FileList.svelte';
	import Settings from '$lib/components/Settings.svelte';
	import Icon from './Icon.svelte';

	const views = [
		{ id: 'browser', icon: 'folder', label: 'Browser' },
		{ id: 'liked', icon: 'heart', label: 'Liked' },
		{ id: 'settings', icon: 'sliders', label: 'Settings' }
	];

	const modes = [
		{ id: 'folders', icon: 'folder', label: 'Folders' },
		{ id: 'artists', icon: 'users', label: 'Artists' },
		{ id: 'albums', icon: 'album', label: 'Albums' },
		{ id: 'allsongs', icon: 'list', label: 'All Songs' }
	];

	let crumb = $derived.by(() => {
		if (app.browseMode === 'folders') {
			const rel = app.cwd.startsWith(app.root) ? app.cwd.slice(app.root.length) : app.cwd;
			const seg = rel.split('/').filter(Boolean).pop();
			return seg || app.root.split('/').filter(Boolean).pop() || 'Library';
		}
		if ((app.browseMode === 'artists' || app.browseMode === 'albums') && app.groupLevel === 1) {
			return app.currentGroup;
		}
		return 'Library';
	});
</script>

<div class="sidebar">
	<div class="tabs">
		{#each views as v (v.id)}
			<button class="tab" class:active={app.view === v.id} onclick={() => setView(v.id)}>
				<Icon name={v.icon} size={13} />
				<span>{v.label}</span>
			</button>
		{/each}
	</div>

	{#if app.view === 'browser'}
		<div class="sep"></div>
		<div class="crumb-row">
			<button class="back" onclick={back}><Icon name="chevron-left" size={16} /></button>
			<Icon name="chevron-right" size={11} />
			<span class="crumb">{crumb}</span>
		</div>
		<div class="sep"></div>
		<div class="tabs">
			{#each modes as m (m.id)}
				<button
					class="tab"
					class:active={app.browseMode === m.id}
					onclick={() => setBrowseMode(m.id)}
				>
					<Icon name={m.icon} size={13} />
					<span>{m.label}</span>
				</button>
			{/each}
		</div>
	{/if}

	<div class="body">
		{#if app.view === 'settings'}
			<Settings />
		{:else}
			<FileList />
		{/if}
	</div>
</div>

<style>
	.sidebar {
		background: var(--sidebar-bg);
		border: 1px solid var(--line);
		height: 100%;
		display: flex;
		flex-direction: column;
		padding: 34px 10px 10px;
		overflow: hidden;
		box-sizing: border-box;
		gap: 8px;
	}

	.tabs {
		display: flex;
		height: 32px;
		gap: 0;
		flex-shrink: 0;
	}

	.tab {
		flex: 1;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 5px;
		font-size: 14px;
		background: transparent;
		color: var(--text-dim);
		border: 1px solid var(--line);
		border-radius: 0;
		cursor: pointer;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		padding: 0 4px;
	}

	.tab + .tab {
		border-left: none;
	}

	.tab:hover:not(.active) {
		background: var(--hover);
		color: var(--text);
	}

	.tab.active {
		background: var(--accent-soft);
		color: var(--accent);
		border: 1px solid var(--accent);
	}

	.sep {
		height: 1px;
		background: var(--line);
		margin: 0;
		flex-shrink: 0;
	}

	.crumb-row {
		display: flex;
		align-items: center;
		gap: 6px;
		height: 24px;
	}

	.back {
		background: transparent;
		border: none;
		color: var(--text-dim);
		font-size: 18px;
		line-height: 1;
		cursor: pointer;
		padding: 0 6px;
		border-radius: 0;
	}

	.back:hover {
		background: var(--hover);
		color: var(--text);
	}

	.crumb {
		font-size: 14px;
		color: var(--text-dim);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		flex: 1;
	}

	.body {
		flex: 1;
		min-height: 0;
		display: flex;
		flex-direction: column;
	}
</style>
