<script>
	let { bars, color = '#9664ff', playing = false } = $props();

	const EMPTY = new Float32Array(64);

	let canvas;
	let parsed = [152, 100, 255];
	let lastColor = '';

	function parseColor(hex) {
		if (hex === lastColor) return;
		lastColor = hex;
		let h = hex.replace('#', '');
		if (h.length === 3) h = h[0] + h[0] + h[1] + h[1] + h[2] + h[2];
		const n = parseInt(h, 16);
		if (!isNaN(n) && h.length === 6) {
			parsed = [(n >> 16) & 255, (n >> 8) & 255, n & 255];
		}
	}

	$effect(() => {
		const ctx = canvas.getContext('2d');
		let raf;
		let w = 0;
		let h = 0;
		let time = 0;
		let rotation = 0;
		let last = performance.now() / 1000;

		const resize = () => {
			const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
			const r = canvas.getBoundingClientRect();
			w = r.width;
			h = r.height;
			canvas.width = Math.max(1, Math.round(w * dpr));
			canvas.height = Math.max(1, Math.round(h * dpr));
			ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
		};

		const ro = new ResizeObserver(resize);
		ro.observe(canvas);
		resize();

		const angleStep = (Math.PI * 2) / 64;

		const draw = (now) => {
			const t = now / 1000;
			const dt = t - last;
			last = t;
			time += dt;

			parseColor(color);
			const [pr, pg, pb] = parsed;
			const b = bars || EMPTY;

			ctx.clearRect(0, 0, w, h);

			const cx = w / 2;
			const cy = h / 2;
			const maxSafe = Math.min(w * 0.45, h * 0.45);
			const inner = maxSafe * 0.35;
			const maxLen = maxSafe * 0.65;

			ctx.strokeStyle = 'rgb(80,80,80)';
			ctx.lineWidth = 2;
			ctx.beginPath();
			ctx.arc(cx, cy, inner, 0, Math.PI * 2);
			ctx.stroke();

			if (playing) rotation = time * 0.5;

			ctx.lineCap = 'butt';
			ctx.lineWidth = 12;
			for (let i = 0; i < 64; i++) {
				let m = b[i];
				if (m < 0) m = 0;
				else if (m > 1) m = 1;
				const barLen = m * maxLen;
				const angle = i * angleStep + rotation;
				const ca = Math.cos(angle);
				const sa = Math.sin(angle);
				const sx = cx + ca * inner;
				const sy = cy + sa * inner;
				const ex = cx + ca * (inner + barLen);
				const ey = cy + sa * (inner + barLen);
				const brightness = 0.3 + m * 0.7;
				const r = Math.max(30, pr * brightness);
				const g = Math.max(30, pg * brightness);
				const bl = Math.max(30, pb * brightness);
				ctx.strokeStyle = `rgb(${r | 0},${g | 0},${bl | 0})`;
				ctx.beginPath();
				ctx.moveTo(sx, sy);
				ctx.lineTo(ex, ey);
				ctx.stroke();
			}

			raf = requestAnimationFrame(draw);
		};

		raf = requestAnimationFrame(draw);

		return () => {
			cancelAnimationFrame(raf);
			ro.disconnect();
		};
	});
</script>

<canvas bind:this={canvas}></canvas>

<style>
	canvas {
		width: 100%;
		height: 100%;
		display: block;
	}
</style>
