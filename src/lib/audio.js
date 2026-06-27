import { convertFileSrc } from '@tauri-apps/api/core';

class AudioEngine {
	constructor() {
		this.audio = null;
		this.ctx = null;
		this.analyser = null;
		this.source = null;
	}

	init() {
		if (this.ctx) return;
		this.audio = new Audio();
		this.audio.crossOrigin = 'anonymous';
		this.audio.preload = 'auto';
		this.ctx = new (window.AudioContext || window.webkitAudioContext)();
		this.analyser = this.ctx.createAnalyser();
		this.analyser.fftSize = 4096;
		this.analyser.smoothingTimeConstant = 0;
		this.analyser.minDecibels = -90;
		this.analyser.maxDecibels = -10;
		this.source = this.ctx.createMediaElementSource(this.audio);
		this.source.connect(this.analyser);
		this.analyser.connect(this.ctx.destination);
	}

	async load(path) {
		this.init();
		this.audio.src = convertFileSrc(path);
		this.audio.load();
	}

	async play() {
		this.init();
		if (this.ctx.state === 'suspended') await this.ctx.resume();
		await this.audio.play();
	}

	pause() {
		if (this.audio) this.audio.pause();
	}

	seek(seconds) {
		if (this.audio) this.audio.currentTime = seconds;
	}

	setVolume(v) {
		if (this.audio) this.audio.volume = v;
	}

	setLoop(on) {
		if (this.audio) this.audio.loop = on;
	}

	get currentTime() {
		return this.audio ? this.audio.currentTime : 0;
	}

	get duration() {
		return this.audio && isFinite(this.audio.duration) ? this.audio.duration : 0;
	}

	async computeWaveform(path, samples = 256) {
		try {
			const res = await fetch(convertFileSrc(path));
			const buf = await res.arrayBuffer();
			const OAC = window.OfflineAudioContext || window.webkitOfflineAudioContext;
			const oac = new OAC(1, 1, 8000);
			const decoded = await oac.decodeAudioData(buf);
			const data = decoded.getChannelData(0);
			const block = Math.floor(data.length / samples) || 1;
			const out = new Array(samples).fill(0);
			for (let i = 0; i < samples; i++) {
				let max = 0;
				const start = i * block;
				for (let j = 0; j < block; j++) {
					const v = Math.abs(data[start + j] || 0);
					if (v > max) max = v;
				}
				out[i] = Math.min(1, max);
			}
			return out;
		} catch {
			return new Array(samples).fill(0.5);
		}
	}
}

export const engine = new AudioEngine();
