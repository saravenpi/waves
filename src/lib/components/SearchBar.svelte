<script>
	import { app, openSearch, closeSearch, setSearch, playQueue } from '$lib/state.svelte.js';
	import Icon from './Icon.svelte';

	let inputEl = $state(null);

	$effect(() => {
		if (app.searchOpen && inputEl) inputEl.focus();
	});

	function onInput(e) {
		setSearch(e.target.value);
	}

	function pick(i) {
		playQueue(app.searchResults, i);
		closeSearch();
		if (inputEl) inputEl.blur();
	}

	function fmt(s) {
		if (!s || !isFinite(s)) return '';
		const m = Math.floor(s / 60);
		const sec = Math.floor(s % 60);
		return `${m}:${sec.toString().padStart(2, '0')}`;
	}
</script>

<div class="search">
	<div class="field">
		<span class="icon"><Icon name="search" size={15} /></span>
		<input
			bind:this={inputEl}
			value={app.searchQuery}
			oninput={onInput}
			onfocus={openSearch}
			placeholder="Search files..."
			spellcheck="false"
		/>
	</div>

	{#if app.searchOpen && app.searchResults.length}
		<ul class="results">
			{#each app.searchResults as r, i}
				<li class:sel={i === app.searchSelected}>
					<button onclick={() => pick(i)}>
						<span class="t">{r.title}</span>
						<span class="a">{r.artist}</span>
						<span class="d">{fmt(r.duration)}</span>
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</div>

<style>
	.search {
		position: relative;
		padding: 0 0 12px;
	}
	.field {
		display: flex;
		align-items: center;
		gap: 8px;
		border: 1px solid var(--accent);
		padding: 8px 12px;
		background: transparent;
	}
	.icon {
		color: var(--text-dim);
		font-size: 15px;
	}
	input {
		flex: 1;
		background: none;
		border: none;
		outline: none;
		color: var(--text);
		font-size: 15px;
	}
	input::placeholder {
		color: var(--text-muted);
	}
	.results {
		position: absolute;
		left: 0;
		right: 0;
		top: 100%;
		z-index: 30;
		list-style: none;
		background: var(--sidebar-bg);
		border: 1px solid var(--line);
		margin-top: -8px;
	}
	.results li button {
		display: flex;
		align-items: center;
		gap: 12px;
		width: 100%;
		padding: 9px 12px;
		text-align: left;
	}
	.results li.sel,
	.results li button:hover {
		background: var(--accent-soft);
	}
	.t {
		flex: 1;
		color: var(--text);
		font-size: 14px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.a {
		color: var(--text-dim);
		font-size: 12px;
	}
	.d {
		color: var(--text-faint);
		font-size: 12px;
	}
</style>
