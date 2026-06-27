export const BANDS = 64;
const FMIN = 20;
const FMAX = 20000;

export class Spectrum {
	constructor() {
		this.bars = new Float32Array(BANDS);
		this.freqData = null;
		this.ranges = null;
	}

	ensure(analyser) {
		const bins = analyser.frequencyBinCount;
		if (!this.freqData || this.freqData.length !== bins) {
			this.freqData = new Uint8Array(bins);
		}
		if (!this.ranges) {
			const nyquist = analyser.context.sampleRate / 2;
			const freqPerBin = nyquist / bins;
			this.ranges = [];
			for (let i = 0; i < BANDS; i++) {
				const fmin = FMIN * Math.pow(FMAX / FMIN, i / BANDS);
				const fmax = FMIN * Math.pow(FMAX / FMIN, (i + 1) / BANDS);
				let s = Math.floor(fmin / freqPerBin);
				let e = Math.min(Math.floor(fmax / freqPerBin), bins);
				if (e <= s) e = s + 1;
				this.ranges.push([s, e]);
			}
		}
	}

	update(analyser, playing) {
		if (!analyser || !playing) {
			for (let i = 0; i < BANDS; i++) {
				this.bars[i] = Math.max(0, this.bars[i] - 0.01);
			}
			return this.bars;
		}
		this.ensure(analyser);
		analyser.getByteFrequencyData(this.freqData);
		for (let i = 0; i < BANDS; i++) {
			const [s, e] = this.ranges[i];
			let sum = 0;
			let n = 0;
			for (let b = s; b < e; b++) {
				sum += this.freqData[b];
				n++;
			}
			const norm = n > 0 ? sum / n / 255 : 0;
			const old = this.bars[i];
			let v = norm > old ? old * 0.4 + norm * 0.6 : old * 0.15 + norm * 0.85;
			const gravity = v < 0.05 ? 0.01 : 0.005;
			v = Math.max(0, v - gravity);
			if (v < 0.001) v = 0;
			this.bars[i] = v;
		}
		return this.bars;
	}
}
