<script>
	let { bars, color = '#9664ff', playing = false } = $props();

	const EMPTY = new Float32Array(64);
	const N = 160;
	const COLS = 16;

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
		let last = performance.now() / 1000;

		const px = new Float32Array(N);
		const py = new Float32Array(N);
		const vx = new Float32Array(N);
		const vy = new Float32Array(N);
		const fx = new Float32Array(N);
		const fy = new Float32Array(N);
		const band = new Int32Array(N);
		const mag = new Float32Array(N);

		for (let i = 0; i < N; i++) band[i] = Math.floor((i * 64) / N);

		const initParticles = () => {
			const paddingX = w * 0.1;
			const paddingY = h * 0.1;
			const usableW = w - 2 * paddingX;
			const usableH = h - 2 * paddingY;
			const spacingX = usableW / 15;
			const spacingY = usableH / 9;
			for (let i = 0; i < N; i++) {
				const col = i % COLS;
				const row = Math.floor(i / COLS);
				px[i] =
					paddingX + col * spacingX + (Math.random() * 2 - 1) * spacingX * 0.3;
				py[i] =
					paddingY + row * spacingY + (Math.random() * 2 - 1) * spacingY * 0.3;
				vx[i] = 0;
				vy[i] = 0;
			}
		};

		let initW = 0;
		let initH = 0;
		const resize = () => {
			const dpr = window.devicePixelRatio || 1;
			const r = canvas.getBoundingClientRect();
			w = r.width;
			h = r.height;
			canvas.width = Math.max(1, Math.round(w * dpr));
			canvas.height = Math.max(1, Math.round(h * dpr));
			ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
			if (
				initW === 0 ||
				Math.abs(w - initW) > initW * 0.1 ||
				Math.abs(h - initH) > initH * 0.1
			) {
				initW = w;
				initH = h;
				initParticles();
			}
		};

		const ro = new ResizeObserver(resize);
		ro.observe(canvas);
		resize();

		const TWO_PI = Math.PI * 2;

		const draw = (now) => {
			const t = now / 1000;
			let dt = t - last;
			last = t;
			if (dt > 0.033) dt = 0.033;
			time += dt;

			parseColor(color);
			const [pr, pg, pb] = parsed;
			const b = bars || EMPTY;

			ctx.clearRect(0, 0, w, h);
			ctx.fillStyle = '#000';
			ctx.fillRect(0, 0, w, h);

			const cx = w / 2;
			const cy = h / 2;

			let sum = 0;
			for (let i = 0; i < 64; i++) sum += b[i];
			const avgMag = sum / 64;

			for (let i = 0; i < N; i++) {
				fx[i] = 0;
				fy[i] = 0;
				mag[i] = b[band[i]];
			}

			for (let i = 0; i < N; i++) {
				const dxc = cx - px[i];
				const dyc = cy - py[i];
				const distC = Math.hypot(dxc, dyc);
				if (distC > 0) {
					const force = 20 * (distC / w);
					fx[i] += (dxc / distC) * force;
					fy[i] += (dyc / distC) * force;
				}
			}

			for (let i = 0; i < N; i++) {
				const xi = px[i];
				const yi = py[i];
				const bandI = band[i];
				const magI = mag[i];
				for (let j = i + 1; j < N; j++) {
					let dx = px[j] - xi;
					let dy = py[j] - yi;
					let distance = Math.hypot(dx, dy);
					if (distance < 20) distance = 20;
					const ux = dx / distance;
					const uy = dy / distance;

					if (distance < 40) {
						const strength = 300 * (1 - distance / 40);
						fx[i] -= ux * strength;
						fy[i] -= uy * strength;
						fx[j] += ux * strength;
						fy[j] += uy * strength;
					}

					const magJ = mag[j];
					const combined = (magI + magJ) * 0.5;
					const attractionStrength = playing ? 800 * (1 + combined * 0.5) : 800;
					const fmag = attractionStrength / (distance * distance);
					fx[i] += ux * fmag;
					fy[i] += uy * fmag;
					fx[j] -= ux * fmag;
					fy[j] -= uy * fmag;

					if (distance < 200) {
						const bandJ = band[j];
						const pulseI = Math.sin(time * 4 + bandI * 0.1) * magI;
						const pulseJ = Math.sin(time * 4 + bandJ * 0.1) * magJ;
						for (let seg = 0; seg < 8; seg++) {
							const tStart = seg / 8;
							const tEnd = (seg + 1) / 8;
							const tMid = (seg + 0.5) / 8;
							const waveI = Math.sin(TWO_PI * (tMid - time * 2)) * pulseI;
							const waveJ = Math.sin(TWO_PI * (1 - tMid - time * 2)) * pulseJ;
							const pulse = Math.abs((waveI + waveJ) * 0.5);
							const alpha =
								Math.min(
									255,
									(1 - distance / 200) *
										100 *
										(1 + avgMag * 2 + combined * 1.5 + pulse * 3)
								) / 255;
							const lw = 1.5 + combined * 2 + pulse * 3;
							ctx.strokeStyle = `rgba(${pr},${pg},${pb},${alpha})`;
							ctx.lineWidth = lw;
							ctx.beginPath();
							ctx.moveTo(xi + dx * tStart, yi + dy * tStart);
							ctx.lineTo(xi + dx * tEnd, yi + dy * tEnd);
							ctx.stroke();
						}
					}
				}
			}

			if (playing) {
				for (let i = 0; i < N; i++) {
					const bd = band[i];
					const m = mag[i];
					fx[i] += Math.sin(time * 3 + bd) * m * 800;
					fy[i] += Math.cos(time * 2.5 + bd) * m * 800;
				}
			}

			for (let i = 0; i < N; i++) {
				vx[i] += fx[i] * dt;
				vy[i] += fy[i] * dt;
				vx[i] *= 0.95;
				vy[i] *= 0.95;
				px[i] += vx[i] * dt;
				py[i] += vy[i] * dt;

				if (px[i] < 0) {
					px[i] = 0;
					vx[i] = -vx[i] * 0.7;
				} else if (px[i] > w) {
					px[i] = w;
					vx[i] = -vx[i] * 0.7;
				}
				if (py[i] < 0) {
					py[i] = 0;
					vy[i] = -vy[i] * 0.7;
				} else if (py[i] > h) {
					py[i] = h;
					vy[i] = -vy[i] * 0.7;
				}
			}

			for (let i = 0; i < N; i++) {
				const m = mag[i];
				const radius = 2.5 + m * 8;
				let intensity = m;
				if (intensity < 0.3) intensity = 0.3;
				else if (intensity > 1) intensity = 1;

				ctx.fillStyle = `rgba(${pr},${pg},${pb},${(m * 80) / 255})`;
				ctx.beginPath();
				ctx.arc(px[i], py[i], radius * 1.5, 0, TWO_PI);
				ctx.fill();

				const r = Math.max(50, pr * intensity);
				const g = Math.max(50, pg * intensity);
				const bl = Math.max(50, pb * intensity);
				ctx.fillStyle = `rgb(${r | 0},${g | 0},${bl | 0})`;
				ctx.beginPath();
				ctx.arc(px[i], py[i], radius, 0, TWO_PI);
				ctx.fill();
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
