<script>
	import Icon from './Icon.svelte';

	let { x = 0, y = 0, items = [], onclose } = $props();

	let el = $state(null);
	let px = $state(0);
	let py = $state(0);

	$effect(() => {
		if (!el) return;
		const rect = el.getBoundingClientRect();
		let nx = x;
		let ny = y;
		if (nx + rect.width > window.innerWidth) nx = Math.max(0, window.innerWidth - rect.width - 4);
		if (ny + rect.height > window.innerHeight) ny = Math.max(0, window.innerHeight - rect.height - 4);
		px = nx;
		py = ny;
	});

	$effect(() => {
		const onpointer = (e) => {
			if (el && el.contains(e.target)) return;
			onclose?.();
		};
		const onkey = (e) => {
			if (e.key === 'Escape') onclose?.();
		};
		const onscroll = () => onclose?.();
		window.addEventListener('pointerdown', onpointer, true);
		window.addEventListener('keydown', onkey, true);
		window.addEventListener('scroll', onscroll, true);
		return () => {
			window.removeEventListener('pointerdown', onpointer, true);
			window.removeEventListener('keydown', onkey, true);
			window.removeEventListener('scroll', onscroll, true);
		};
	});

	function run(item) {
		item.action?.();
		onclose?.();
	}
</script>

<div class="menu" bind:this={el} style="left: {px}px; top: {py}px;">
	{#each items as item}
		{#if item.separator}
			<div class="sep"></div>
		{:else}
			<button class="item" class:danger={item.danger} onclick={() => run(item)}>
				{#if item.icon}
					<span class="ico"><Icon name={item.icon} size={14} /></span>
				{:else}
					<span class="ico"></span>
				{/if}
				<span class="label">{item.label}</span>
			</button>
		{/if}
	{/each}
</div>

<style>
	.menu {
		position: fixed;
		z-index: 200;
		min-width: 180px;
		padding: 4px 0;
		background: var(--sidebar-bg);
		border: 1px solid var(--line);
		border-radius: 0;
		font-family: 'Undefined', monospace;
		font-size: 13px;
	}

	.item {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 7px 14px;
		background: none;
		border: none;
		border-radius: 0;
		text-align: left;
		font-family: 'Undefined', monospace;
		font-size: 13px;
		color: var(--text);
		cursor: pointer;
	}

	.item:hover {
		background: var(--hover);
	}

	.item.danger {
		color: #ff6464;
	}

	.ico {
		flex: 0 0 14px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}

	.label {
		flex: 1 1 auto;
	}

	.sep {
		height: 1px;
		margin: 4px 0;
		background: var(--line);
	}
</style>
