<script>
	import { app, togglePlay, next, prev, toggleLoop, toggleFavoritePath, isLiked } from '$lib/state.svelte.js';
	import Waveform from './Waveform.svelte';
	import Icon from './Icon.svelte';

	function fmt(s) {
		const total = Math.floor(s || 0);
		const m = Math.floor(total / 60);
		const sec = total % 60;
		return `${m}:${sec.toString().padStart(2, '0')}`;
	}

	let liked = $derived(app.current ? isLiked(app.current.path) : false);
</script>

{#if app.current}
	<div class="bar">
		<div class="cover">
			{#if app.cover}
				<img src={app.cover} alt="" />
			{:else}
				<div class="placeholder"><Icon name="music" size={24} /></div>
			{/if}
		</div>

		<div class="main">
			<div class="top">
				<div class="meta">
					<div class="title">{app.current.title}</div>
					<div class="artist">{app.current.artist || 'Unknown artist'}</div>
				</div>
				<div class="controls">
					<button
						class="ctrl"
						class:on={liked}
						onclick={() => toggleFavoritePath(app.current.path, app.current.name)}
						aria-label="Favorite"
					>
						<Icon name="heart" size={18} />
					</button>
					<button class="ctrl" onclick={prev} aria-label="Previous"><Icon name="prev" size={18} /></button>
					<button class="ctrl" onclick={togglePlay} aria-label="Play/pause">
						<Icon name={app.playing ? 'pause' : 'play'} size={18} />
					</button>
					<button class="ctrl" onclick={next} aria-label="Next"><Icon name="next" size={18} /></button>
					<button class="ctrl loop" class:on={app.loop} onclick={toggleLoop} aria-label="Loop">
						<Icon name="repeat" size={18} />
					</button>
				</div>
			</div>

			<div class="wave">
				<Waveform />
			</div>

			<div class="time">
				<span class="cur">{fmt(app.currentTime)}</span>
				<span class="total">{fmt(app.duration)}</span>
			</div>
		</div>
	</div>
{:else}
	<div class="bar empty">No track playing</div>
{/if}

<style>
	.bar {
		display: flex;
		align-items: stretch;
		gap: 12px;
		padding: 10px 12px;
		font-family: 'Undefined', monospace;
		border-top: 1px solid var(--line);
		background: var(--panel);
	}

	.bar.empty {
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-muted);
		font-size: 16px;
		min-height: 64px;
	}

	.cover {
		flex: 0 0 auto;
		width: 64px;
		height: 64px;
	}

	.cover img {
		width: 64px;
		height: 64px;
		object-fit: cover;
		border-radius: 4px;
		display: block;
	}

	.placeholder {
		width: 64px;
		height: 64px;
		border-radius: 4px;
		background: var(--input-bg);
		border: 1px solid var(--line);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 22px;
		color: var(--text-muted);
		opacity: 0.6;
	}

	.main {
		flex: 1 1 auto;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.top {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 12px;
	}

	.meta {
		min-width: 0;
	}

	.title {
		font-size: 18px;
		color: var(--text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.artist {
		font-size: 13px;
		color: var(--text-dim);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.controls {
		flex: 0 0 auto;
		display: flex;
		gap: 6px;
	}

	.ctrl {
		width: 34px;
		height: 34px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 18px;
		background: transparent;
		color: var(--text);
		border-radius: 0;
	}

	.ctrl:hover {
		box-shadow: inset 0 0 0 1px var(--line);
	}

	.ctrl.on {
		color: var(--accent);
	}

	.loop {
		color: var(--text-dim);
	}

	.loop.on {
		color: var(--accent);
	}

	.wave {
		height: 70px;
		width: 100%;
	}

	.time {
		display: flex;
		justify-content: space-between;
		font-size: 13px;
	}

	.cur {
		color: var(--text);
	}

	.total {
		color: var(--text-dim);
	}
</style>
