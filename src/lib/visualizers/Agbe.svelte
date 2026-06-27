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

	function avg(b, from, to) {
		let s = 0;
		for (let i = from; i < to; i++) s += b[i];
		return s / (to - from);
	}

	$effect(() => {
		const ctx = canvas.getContext('2d');
		let raf;
		let w = 0;
		let h = 0;
		let time = 0;
		let last = performance.now() / 1000;

		const resize = () => {
			const dpr = window.devicePixelRatio || 1;
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

		const TWO_PI = Math.PI * 2;
		const angles = new Float32Array(64);
		const cosT = new Float32Array(64);
		const sinT = new Float32Array(64);
		for (let i = 0; i < 64; i++) {
			angles[i] = (i / 64) * TWO_PI;
			cosT[i] = Math.cos(angles[i]);
			sinT[i] = Math.sin(angles[i]);
		}

		const draw = (now) => {
			const t = now / 1000;
			const dt = t - last;
			last = t;
			if (playing) time += dt;

			parseColor(color);
			const [pr, pg, pb] = parsed;
			const b = bars || EMPTY;

			ctx.clearRect(0, 0, w, h);

			const cx = w / 2;
			const cy = h / 2;
			const maxRadius = Math.min(w * 0.35, h * 0.35);

			const bandLow = avg(b, 0, 4);
			const bandMid1 = avg(b, 4, 12);
			const bandMid2 = avg(b, 16, 32);
			const bandHigh = avg(b, 48, 64);
			const midMag = bandMid2;

			ctx.lineWidth = 2.5;
			for (let r = 0; r < 100; r++) {
				const ringProgress = r / 100;
				const baseRadius = maxRadius * (0.2 + ringProgress * 0.7);

				let ringMag;
				if (r < 25) ringMag = bandLow;
				else if (r < 50) ringMag = bandMid1;
				else if (r < 75) ringMag = bandMid2;
				else ringMag = bandHigh;

				const radius = baseRadius * ringMag * 2.0;

				ctx.beginPath();
				for (let i = 0; i < 64; i++) {
					const deformation = b[i] * 30;
					const wobble = Math.sin(angles[i] * 3 + time * 2) * 15 * midMag;
					const finalRadius = radius + deformation + wobble;
					const px = cx + cosT[i] * finalRadius;
					const py = cy + sinT[i] * finalRadius;
					if (i === 0) ctx.moveTo(px, py);
					else ctx.lineTo(px, py);
				}
				ctx.closePath();

				const hueShift = (time * 0.2 + ringProgress) % 1;
				const baseBrightness = 0.2 + ringMag * 0.8;
				const brightnessVar = 0.8 + hueShift * 0.4;
				const boost = ringMag > 0.5 ? 1 + (ringMag - 0.5) : 1;
				const finalIntensity = Math.min(baseBrightness * brightnessVar * boost, 1.5);
				const cr = Math.min(pr * finalIntensity, 255);
				const cg = Math.min(pg * finalIntensity, 255);
				const cb = Math.min(pb * finalIntensity, 255);
				const alpha = (200 * (1 - ringProgress * 0.5)) / 255;
				ctx.strokeStyle = `rgba(${cr | 0},${cg | 0},${cb | 0},${alpha})`;
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
