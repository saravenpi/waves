<script>
	import { onMount } from 'svelte';
	import { app, spectrumBars, initApp, tick, applyConfig } from '$lib/state.svelte.js';
	import { handleKey } from '$lib/keys.js';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import SearchBar from '$lib/components/SearchBar.svelte';
	import NowPlaying from '$lib/components/NowPlaying.svelte';
	import StatusBar from '$lib/components/StatusBar.svelte';
	import MetadataEditor from '$lib/components/MetadataEditor.svelte';
	import Prompt from '$lib/components/Prompt.svelte';
	import Confirm from '$lib/components/Confirm.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import Spectrum from '$lib/visualizers/Spectrum.svelte';
	import CircleSpectrum from '$lib/visualizers/CircleSpectrum.svelte';
	import Agbe from '$lib/visualizers/Agbe.svelte';
	import Dots from '$lib/visualizers/Dots.svelte';

	const VIZ = {
		spectrum: Spectrum,
		circle_spectrum: CircleSpectrum,
		agbe: Agbe,
		dots: Dots
	};

	let Viz = $derived(VIZ[app.config.animation_type] || Spectrum);

	let dragging = false;

	let fsControls = $state(true);
	let fsTimer;
	function fsMove() {
		fsControls = true;
		clearTimeout(fsTimer);
		fsTimer = setTimeout(() => (fsControls = false), 4000);
	}
	function enterFs() {
		app.fullscreenViz = true;
		fsMove();
	}
	function exitFs() {
		app.fullscreenViz = false;
		clearTimeout(fsTimer);
		fsControls = true;
	}

	function startResize(e) {
		dragging = true;
		e.preventDefault();
		const onMove = (ev) => {
			if (!dragging) return;
			const x = ev.clientX;
			const w =
				app.config.sidebar_position === 'left'
					? x
					: window.innerWidth - x;
			app.config.sidebar_width = Math.min(800, Math.max(250, w));
		};
		const onUp = () => {
			dragging = false;
			window.removeEventListener('pointermove', onMove);
			window.removeEventListener('pointerup', onUp);
			applyConfig({ sidebar_width: app.config.sidebar_width });
		};
		window.addEventListener('pointermove', onMove);
		window.addEventListener('pointerup', onUp);
	}

	onMount(() => {
		initApp();
		let raf;
		const loop = () => {
			tick();
			raf = requestAnimationFrame(loop);
		};
		raf = requestAnimationFrame(loop);
		return () => cancelAnimationFrame(raf);
	});
</script>

<svelte:window onkeydown={handleKey} />

<div
	class="app"
	style="border-radius:{app.config.window_corner_radius}px"
	class:right={app.config.sidebar_position === 'right'}
>
	<div class="dragstrip" data-tauri-drag-region></div>

	{#if !app.fullscreenViz}
	<div class="body">
		<div class="sidebar-wrap" style="width:{app.config.sidebar_width}px">
			<Sidebar />
		</div>
		<div
			class="resizer"
			onpointerdown={startResize}
			role="separator"
			aria-orientation="vertical"
			tabindex="-1"
		></div>

		<section class="content">
			<SearchBar />
			<div class="stage">
				{#if app.config.animation && !app.fullscreenViz}
					<Viz bars={spectrumBars} color={app.config.primary_color} playing={app.playing} />
				{/if}
				<button class="fs-enter" onclick={enterFs} aria-label="Fullscreen animation" title="Fullscreen">
					<Icon name="scale" size={16} />
				</button>
			</div>
			<NowPlaying />
		</section>
	</div>

	{#if app.config.show_status_bar}
		<StatusBar />
	{/if}
	{/if}

	<MetadataEditor />
	<Prompt />
	<Confirm />
</div>

{#if app.fullscreenViz}
	<div class="fs-overlay" class:idle={!fsControls} onmousemove={fsMove} role="presentation">
		<Viz bars={spectrumBars} color={app.config.primary_color} playing={app.playing} />
		<button class="fs-close" onclick={exitFs} aria-label="Exit fullscreen">
			<Icon name="close" size={20} />
		</button>
	</div>
{/if}

<style>
	.app {
		position: relative;
		display: flex;
		flex-direction: column;
		height: 100vh;
		overflow: hidden;
		background: var(--bg);
	}
	.dragstrip {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: 30px;
		z-index: 5;
	}
	.body {
		flex: 1;
		display: flex;
		min-height: 0;
	}
	.app.right .body {
		flex-direction: row-reverse;
	}
	.sidebar-wrap {
		flex-shrink: 0;
		height: 100%;
		min-width: 250px;
	}
	.resizer {
		width: 5px;
		cursor: col-resize;
		flex-shrink: 0;
		background: transparent;
	}
	.resizer:hover {
		background: var(--line);
	}
	.content {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-width: 0;
		padding: 34px 30px 0;
	}
	.stage {
		flex: 1;
		min-height: 0;
		display: flex;
		position: relative;
	}
	.fs-enter {
		position: absolute;
		top: 10px;
		right: 10px;
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-dim);
		opacity: 0.45;
		transition: opacity 0.18s var(--ease), color 0.18s var(--ease);
		z-index: 3;
	}
	.fs-enter:hover {
		opacity: 1;
		color: var(--text);
	}

	.fs-overlay {
		position: fixed;
		inset: 0;
		z-index: 60;
		background: var(--bg);
		display: flex;
	}
	.fs-overlay.idle {
		cursor: none;
	}
	.fs-close {
		position: absolute;
		top: 16px;
		right: 16px;
		width: 42px;
		height: 42px;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text);
		background: rgba(0, 0, 0, 0.35);
		z-index: 61;
		opacity: 1;
		transition: opacity 0.6s ease;
	}
	.fs-close:hover {
		color: var(--accent);
	}
	.fs-overlay.idle .fs-close {
		opacity: 0;
		pointer-events: none;
	}
</style>
