# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WAVES is a cross-platform GUI music player written in Rust using the eframe/egui framework. It runs on macOS, Linux, and Windows. It features a Miller column file browser (similar to macOS Finder), real-time audio visualization with FFT-based spectrum analysis, waveform displays, and file/folder icons for easy navigation.

![WAVES Demo](waves_demo.png)

### Platform Support
- **macOS**: Full support with native app bundle
- **Linux**: Full support with desktop integration
- **Windows**: Full support with file associations

### Command-Line Arguments
- `waves` - Opens in system audio directory (~/Music on macOS/Linux, Music library on Windows)
- `waves /path/to/folder` - Opens in specified folder
- `waves /path/to/audio.mp3` - Opens folder containing the file and plays it automatically

## Build and Run Commands

### Development
- `make build` - Build in debug mode and copy binary to project root
- `make run` or `cargo run` - Build and run in debug mode
- `cargo build` - Standard debug build (binary in target/debug/ or target/debug/waves.exe on Windows)

### Release
- `make release` - Build optimized release version and copy to project root
- `cargo build --release` - Standard release build (binary in target/release/ or target/release/waves.exe on Windows)

### Platform-Specific Installation

**macOS:**
- `./install-macos.sh` - Install WAVES as a macOS application in /Applications
  - Creates proper app bundle structure
  - Converts waves_logo.png to ICNS format for app icon
  - Registers file associations for audio files (mp3, wav, flac, ogg, m4a)
  - Enables opening files/folders from Finder
  - Makes WAVES available in Spotlight

**Linux:**
- `chmod +x install-linux.sh && ./install-linux.sh` - Install WAVES on Linux
  - Installs binary to ~/.local/bin/waves
  - Creates desktop entry for application launchers
  - Registers file associations for audio files

**Windows:**
- `.\install-windows.ps1` - Install WAVES on Windows (PowerShell)
  - Installs to %LOCALAPPDATA%\Programs\WAVES
  - Adds to PATH
  - Registers file associations for audio files

### Cleanup
- `make clean` or `cargo clean` - Remove all build artifacts

## Architecture

### Multi-Module Design
The application is organized into logical module groups:

**Core Modules:**
- `src/main.rs` - Entry point, font loading, window setup
- `src/config.rs` - Configuration management (`~/.waves.yml`)
- `src/types.rs` - Core data structures (FileEntry, Column, Favorite, ClipboardOperation, SidebarView)
- `src/favorites.rs` - Favorites persistence (`~/.waves/favorites.yml`)
- `src/metadata.rs` - Metadata extraction for all audio formats
- `src/album_cover.rs` - Album cover extraction and caching
- `src/utils.rs` - Utility functions

**App Module** (`src/app/`):
- `state.rs` - WavesApp struct definition and initialization
- `methods.rs` - Core app methods (update_columns, play_file, etc.)
- `navigation.rs` - Navigation logic (vim keybindings, column selection)
- `playback.rs` - Playback controls (play, pause, seek, volume)
- `search.rs` - File search functionality

**Audio Module** (`src/audio/`):
- `player.rs` - PlayerState struct and playback state management
- `spectrum.rs` - SpectrumCapture wrapper for real-time audio buffer capture
- `waveform.rs` - Waveform extraction and placeholder generation

**UI Module** (`src/ui/`):
- `render.rs` - Main UI rendering (eframe::App implementation)
- `input.rs` - Input handling (keyboard, mouse) and MetadataEditor
- `helpers.rs` - UI helper functions

**File Operations Module** (`src/file_operations/`):
- `browser.rs` - File browser logic and filtering
- `operations.rs` - File operations (copy, cut, paste, delete, rename)
- `search.rs` - SearchResult type and search implementation

### Key Components

**WavesApp** - Main application state containing:
- Miller column file browser (`columns: Vec<Column>`)
- Audio player state wrapped in `Arc<Mutex<Option<PlayerState>>>`
- Waveform and album cover caches with async loading via channels
- FFT spectrum analyzer (`FftPlanner<f32>`, 64 bars)
- Volume control and seeking functionality
- Search system with results and selection state
- Clipboard operations (copy/cut/paste)
- Favorites system with persistence
- Metadata editor for tag editing
- UI state (prompts, context menu, sidebar view, loop mode)

**PlayerState** - Audio playback state:
- rodio `Sink` for audio output
- Custom `SpectrumCapture` wrapper for real-time audio buffer capture
- Waveform data (500 samples representing track amplitude)
- Metadata (title, artist, duration)
- Timing state for seeking (start_time, pause_offset)

**SpectrumCapture** - Custom rodio `Source` wrapper:
- Implements both `Iterator` and `Source` traits
- Captures audio samples into `Arc<Mutex<VecDeque<f32>>>` for FFT analysis
- Maintains circular buffer of last 8192 samples

### Audio Processing Pipeline

1. File decoding: rodio `Decoder` → f32 samples
2. Capture wrapper: `SpectrumCapture` intercepts samples for visualization
3. FFT: 4096-sample Hann-windowed FFT using rustfft
4. Spectrum: 64 logarithmic frequency bands (20Hz-20kHz) with smoothing
5. Display: Real-time bars with gravity effect

### Async Resource Loading

The app uses Rust channels for background resource loading:

**Waveform Loading:**
- Background thread extracts waveforms when files are played
- Results sent via `waveform_sender` channel
- Main UI polls `waveform_receiver` and updates cache
- Placeholder waveform shown until real one arrives

**Album Cover Loading:**
- Similar channel-based approach for album art extraction
- Covers extracted from audio file metadata
- Cached as egui `TextureHandle` for efficient rendering
- Background processing prevents UI blocking

### Navigation System

Miller column browser with vim-style keybindings:

**Navigation:**
- h/j/k/l: navigate (left/down/up/right)
- Enter/l: select directory or play file
- Tab: cycle through views (File Browser → Favorites → Settings → File Browser)

**Playback:**
- Space: pause/resume
- Arrow Left/Right: previous/next track
- Arrow Up/Down: increase/decrease volume

**File Operations:**
- y: copy (yank) selected file/folder (toggle on/off)
- x: cut selected file/folder (toggle on/off)
- p: paste - into selected folder or current directory
- r: rename selected file/folder
- d: delete selected file/folder (with confirmation)
- n: create new folder
- Escape: cancel clipboard operation

**Organization:**
- f: toggle favorite for selected file/folder
- m: open metadata editor for audio files

**Search:**
- Ctrl+F or /: open search (implementation in UI)

The columns update dynamically: current directory in left column, preview of selected directory in right column.

**File Filtering:** Only folders and audio files (mp3, wav, flac, ogg, m4a) are displayed. All other file types are hidden.

## Technical Details

### Supported Audio Formats
- **Playback**: mp3, wav, flac, ogg, m4a (handled by rodio/symphonia)
- **Metadata Editing**:
  - MP3: Full support (id3)
  - M4A/MP4: Full support (mp4ameta)
  - FLAC: Full support (metaflac)
  - WAV: Full support via manual RIFF LIST INFO chunk parsing and writing
  - OGG/Vorbis/Opus: Full support via Vorbis Comments (oggvorbismeta)

### Waveform Generation
- Extracted asynchronously in background thread when file is played
- 500 samples created by chunking full audio and finding max amplitude per chunk
- Cached in HashMap to avoid re-extraction
- Placeholder waveform shown until real one loads

### FFT Spectrum Analysis
- 4096-sample FFT with Hann window
- 64 logarithmic frequency bands
- Smoothing factors: 0.6 (rising), 0.88 (falling)
- Gravity effect (0.003) for natural bar descent
- dB scale mapping: (magnitude + 1e-10) → dB → normalized [0,1]

### Seeking Implementation
Two approaches:
1. Try `Sink::try_seek()` first (fast but not always supported)
2. Fallback: Stop sink, reload file, use `Source::skip_duration()` (slower but reliable)

Seeking uses pending state pattern: drag updates `pending_seek`, release commits actual seek.

## Customization Notes

### Configuration File
Location:
- **macOS/Linux:** `~/.waves.yml`
- **Windows:** `%USERPROFILE%\.waves.yml`

Available options:
- `animation` (bool): Enable/disable spectrum visualization (default: true)
- `sidebar_position` (string): "left" or "right" (default: left)
- `sidebar_width` (float): Width of the sidebar in pixels (default: 500.0, range: 300.0-800.0)
- `decorations` (bool): Show/hide window title bar (default: true)
- `window_corner_radius` (float): Corner rounding radius in pixels (default: 0.0)
- `default_folder` (string): Default folder to open when no argument provided (supports ~ expansion)
- `show_status_bar` (bool): Show/hide keyboard shortcuts bar at bottom (default: true)
- `primary_color` (string): Hex color for UI accents (default: "#9664FF")
- `window_opacity` (float): Window opacity percentage (default: 100.0)
- `custom_font` (string): Path to custom font file (optional, supports ~ expansion)

Example:
```yaml
animation: true
sidebar_position: right
sidebar_width: 600.0
decorations: false
window_corner_radius: 12.0
default_folder: "~/Music/Musa/"
show_status_bar: false
custom_font: "~/Library/Fonts/MyFont.ttf"
```

### Resizable Sidebar
The sidebar is resizable by dragging the edge. The width is automatically saved to the config file and persists across sessions. Width can be adjusted between 300 and 800 pixels.

### Font
Custom fonts can now be configured via the `custom_font` option in the config file. If not specified, the application uses the system default font. The font path supports tilde expansion for cross-platform compatibility.

### Default Directory
Prioritizes directories in this order:
1. User-configured `default_folder` from config
2. System audio directory (uses `dirs::audio_dir()` for cross-platform support)
3. `~/Music` folder if it exists
4. Current directory as fallback

### UI Styling
- Black background theme
- White text with size 18.0
- Spectrum bars: primary color gradient based on intensity (configurable)
- Waveform: primary color (played), gray (unplayed), white progress line
- Configurable window corner radius and opacity via config file
- File/folder icons:
  - 📁 Folders
  - 🎵 Audio files (only audio files are displayed)
  - ⭐ Favorited items (star prefix)

## Development Notes

### Important Patterns

**Thread Safety:**
- `Arc<Mutex<Option<PlayerState>>>` for shared audio player access
- `Arc<Mutex<VecDeque<f32>>>` for spectrum analysis buffer
- Channel-based communication for async resource loading

**State Management:**
- Single `WavesApp` struct owns all application state
- Separated into logical modules via submodules (app/*, audio/*, ui/*, file_operations/*)
- Config persisted to `~/.waves.yml` (macOS/Linux) or `%USERPROFILE%\.waves.yml` (Windows)
- Favorites persisted to `~/.waves/favorites.yml` (macOS/Linux) or `%USERPROFILE%\.waves\favorites.yml` (Windows)

**Error Handling:**
- Metadata extraction uses panic catching (`catch_unwind`) for robustness
- Fallback mechanisms for seeking (try fast method, fall back to slow)
- Graceful degradation when resources unavailable

**Performance Considerations:**
- Waveforms cached to avoid re-extraction (HashMap lookup)
- Album covers cached as TextureHandle for GPU efficiency
- FFT smoothing prevents visual jitter (separate factors for rise/fall)
- Only audio files shown in browser (filtering at read time)
