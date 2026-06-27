<script>
	import { app, SETTINGS, settingsAdjust } from '$lib/state.svelte.js';

	function selectRow(i) {
		app.settingsSelected = i;
	}

	function activate(i, item) {
		app.settingsSelected = i;
		if (item.type === 'toggle' || item.type === 'cycle') settingsAdjust(1);
	}

	function step(i, dir) {
		app.settingsSelected = i;
		settingsAdjust(dir);
	}
</script>

<div class="list">
	{#each SETTINGS as item, i (item.key)}
		<div
			class="row"
			class:selected={i === app.settingsSelected}
			role="button"
			tabindex="-1"
			onclick={() => activate(i, item)}
			onfocus={() => selectRow(i)}
		>
			<span class="label">{item.label}</span>
			<span class="spacer"></span>

			{#if item.type === 'toggle'}
				<span class="value" class:on={app.config[item.key]}>{app.config[item.key] ? 'ON' : 'OFF'}</span>
			{:else if item.type === 'cycle'}
				<button class="arrow" onclick={(e) => { e.stopPropagation(); step(i, -1); }}>‹</button>
				{#if item.key === 'primary_color'}
					<span class="swatch" style="background: {app.config[item.key]}"></span>
					<span class="value cyc">{app.config[item.key]}</span>
				{:else}
					<span class="value cyc">{app.config[item.key]}</span>
				{/if}
				<button class="arrow" onclick={(e) => { e.stopPropagation(); step(i, 1); }}>›</button>
			{:else if item.type === 'range'}
				<button class="arrow" onclick={(e) => { e.stopPropagation(); step(i, -1); }}>‹</button>
				<span class="pct">{Math.round((app.config[item.key] || 0) * 100)}%</span>
				<span class="track">
					<span class="fill" style="width: {Math.round((app.config[item.key] || 0) * 100)}%"></span>
				</span>
				<button class="arrow" onclick={(e) => { e.stopPropagation(); step(i, 1); }}>›</button>
			{/if}
		</div>
	{/each}
</div>

<style>
	.list {
		flex: 1;
		overflow-y: auto;
		min-height: 0;
		font-family: 'Undefined', monospace;
	}

	.row {
		height: 30px;
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px;
		color: var(--text);
		cursor: pointer;
		border-radius: 0;
	}

	.row:hover:not(.selected) {
		background: var(--hover);
	}

	.row.selected {
		background: var(--accent-soft);
		box-shadow: inset 0 0 0 1px var(--accent);
	}

	.row.selected .label {
		color: var(--accent);
	}

	.label {
		font-size: 14px;
		color: var(--text-dim);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.spacer {
		flex: 1 1 auto;
	}

	.value {
		font-size: 14px;
	}

	.value.on {
		color: var(--accent);
	}

	.value:not(.on):not(.cyc) {
		color: var(--text-muted);
	}

	.value.cyc {
		color: var(--text);
	}

	.swatch {
		width: 12px;
		height: 12px;
		flex: 0 0 auto;
		border: 1px solid var(--line);
		display: inline-block;
	}

	.arrow {
		flex: 0 0 auto;
		width: 18px;
		height: 18px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 14px;
		background: transparent;
		color: var(--text-dim);
		border-radius: 0;
	}

	.arrow:hover {
		color: var(--text);
		box-shadow: inset 0 0 0 1px var(--line);
	}

	.pct {
		font-size: 13px;
		color: var(--text);
		min-width: 36px;
		text-align: right;
	}

	.track {
		flex: 0 0 auto;
		width: 80px;
		height: 4px;
		background: var(--input-bg);
		display: inline-block;
		position: relative;
	}

	.fill {
		position: absolute;
		left: 0;
		top: 0;
		height: 100%;
		background: var(--accent);
	}
</style>
