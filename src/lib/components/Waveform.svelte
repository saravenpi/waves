<script>
	import { app, seekTo } from '$lib/state.svelte.js';

	let canvas = $state(null);
	let box = $state(null);
	let dragging = false;
	let raf = 0;
	let lastData = null;
	let lastW = 0;
	let lastH = 0;
	let lastProg = -1;

	function accent() {
		return getComputedStyle(document.documentElement).getPropertyValue('--accent').trim() || '#9664ff';
	}

	function draw() {
		if (!canvas || !box || document.hidden) return;
		const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
		const w = box.clientWidth;
		const h = box.clientHeight;
		const prog = app.duration > 0 ? app.currentTime / app.duration : 0;
		if (app.waveform === lastData && w === lastW && h === lastH && prog === lastProg) return;
		lastData = app.waveform;
		lastW = w;
		lastH = h;
		lastProg = prog;
		if (canvas.width !== Math.round(w * dpr) || canvas.height !== Math.round(h * dpr)) {
			canvas.width = Math.round(w * dpr);
			canvas.height = Math.round(h * dpr);
		}
		const ctx = canvas.getContext('2d');
		ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		ctx.clearRect(0, 0, w, h);

		const data = app.waveform;
		const mid = h / 2;
		const progress = app.duration > 0 ? Math.min(1, Math.max(0, app.currentTime / app.duration)) : 0;

		if (!data || data.length === 0) {
			ctx.strokeStyle = '#404040';
			ctx.lineWidth = 1;
			ctx.beginPath();
			ctx.moveTo(0, mid);
			ctx.lineTo(w, mid);
			ctx.stroke();
			return;
		}

		const n = data.length;
		const slot = w / n;
		const barW = Math.max(1, slot * 0.4);
		const progX = progress * w;

		for (let i = 0; i < n; i++) {
			const x = i * slot + slot / 2;
			const amp = Math.min(1, Math.max(0, data[i]));
			const half = amp * (h / 2) * 0.9;
			ctx.fillStyle = x <= progX ? '#878787' : '#404040';
			ctx.fillRect(x - barW / 2, mid - half, barW, half * 2);
		}

		ctx.fillStyle = accent();
		ctx.fillRect(progX - 1, 0, 2, h);
	}

	function loop() {
		draw();
		raf = requestAnimationFrame(loop);
	}

	function seekAt(clientX) {
		if (!box || app.duration <= 0) return;
		const rect = box.getBoundingClientRect();
		const frac = Math.min(1, Math.max(0, (clientX - rect.left) / rect.width));
		seekTo(frac * app.duration);
	}

	function onDown(e) {
		dragging = true;
		box.setPointerCapture(e.pointerId);
		seekAt(e.clientX);
	}

	function onMove(e) {
		if (dragging) seekAt(e.clientX);
	}

	function onUp(e) {
		dragging = false;
		if (box.hasPointerCapture(e.pointerId)) box.releasePointerCapture(e.pointerId);
	}

	$effect(() => {
		const ro = new ResizeObserver(() => draw());
		if (box) ro.observe(box);
		raf = requestAnimationFrame(loop);
		return () => {
			cancelAnimationFrame(raf);
			ro.disconnect();
		};
	});
</script>

<div
	class="box"
	bind:this={box}
	role="slider"
	tabindex="-1"
	aria-valuenow={app.currentTime}
	onpointerdown={onDown}
	onpointermove={onMove}
	onpointerup={onUp}
	onpointercancel={onUp}
>
	<canvas bind:this={canvas}></canvas>
</div>

<style>
	.box {
		width: 100%;
		height: 100%;
		cursor: pointer;
		touch-action: none;
	}

	canvas {
		display: block;
		width: 100%;
		height: 100%;
	}
</style>
