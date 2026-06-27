<script>
	import { app } from '$lib/state.svelte.js';
</script>

<div class="status">
	<span>h/j/k/l: navigate</span>
	<span class="sep">|</span>
	<span>ENTER: select/play</span>
	<span class="sep">|</span>
	<span>SPACE: pause</span>
	<span class="sep">|</span>
	<span>←/→: prev/next</span>
	<span class="sep">|</span>
	<span>TAB: view</span>
	<span class="sep">|</span>
	<span>b: mode</span>
	<span class="sep">|</span>
	<span>f: like</span>
	<span class="sep">|</span>
	<span>m: edit</span>
	<span class="sep">|</span>
	<span>/: search</span>
	<span class="sep">|</span>
	<span>↑/↓: vol ({Math.round(app.volume * 100)}%)</span>
</div>

<style>
	.status {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: nowrap;
		overflow: hidden;
		background: var(--statusbar-bg);
		border-top: 1px solid var(--line);
		padding: 6px 16px;
		font-size: 13px;
		color: var(--text);
		white-space: nowrap;
		flex-shrink: 0;
	}
	.sep {
		color: var(--text-muted);
	}
</style>
