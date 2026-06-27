class SoundEngine {
	constructor() {
		this.enabled = true;
		this.volume = 0.04;
		this.startupEnabled = true;
	}

	configure(enabled, volume, startupEnabled) {
		this.enabled = enabled;
		this.volume = volume;
		this.startupEnabled = startupEnabled;
	}

	play(file, volume) {
		try {
			const a = new Audio(`/sounds/${file}`);
			a.volume = Math.min(1, Math.max(0, volume));
			a.play().catch(() => {});
		} catch {}
	}

	cursor() {
		if (this.enabled) this.play('cursor_move.wav', this.volume);
	}

	delete() {
		if (this.enabled) this.play('delete.mp3', this.volume);
	}

	startup() {
		if (this.startupEnabled) this.play('startup.mp3', 0.5);
	}
}

export const sounds = new SoundEngine();
