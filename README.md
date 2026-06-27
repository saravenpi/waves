# Waves Desktop

The Tauri version of [Waves](../) — a local-first music player with real-time
audio visualizers. Built with Tauri 2 + SvelteKit (Svelte 5) + Bun, mirroring
the toile build-and-publish-to-website pipeline.

## Stack

- **Frontend**: SvelteKit (static adapter), Svelte 5 runes, vanilla CSS
- **Audio**: Web Audio API — `<audio>` element + `AnalyserNode` (4096-point FFT)
  feeding canvas visualizers. No Rust DSP needed.
- **Backend (Rust)**: minimal — a `scan_dir` command (recursive folder scan with
  [`lofty`](https://crates.io/crates/lofty) metadata) and `read_cover` (embedded
  album art as a data URL). File access via Tauri's asset protocol.

## Visualizers

Two real-time, 60fps canvas visualizers (log-grouped frequency bins, asymmetric
rise/fall easing, graceful idle states):

- **Bars** — 64 bottom-anchored spectrum bars with gradient + glow.
- **Radial** — 128-spoke mirrored ring that slowly rotates.

## Develop

```bash
bun install
bun run tauri dev
```

## Build + publish

```bash
./release.sh            # Apple Silicon DMG → ../waves-website/static/downloads/Waves-aarch64.dmg
./release.sh --intel    # Intel DMG → Waves-x64.dmg
```

The website serves `Waves-aarch64.dmg` from its `downloads/` folder via a
"Download for Mac" button, exactly like toile.
