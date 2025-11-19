# Waves - Installation Guide

WAVES is a cross-platform GUI music player. Choose your platform below for installation instructions.

## Table of Contents
- [macOS Installation](#macos-installation)
- [Linux Installation](#linux-installation)
- [Windows Installation](#windows-installation)
- [Cross-Platform Configuration](#cross-platform-configuration)

---

## macOS Installation

### First Time Opening (Important!)

If you downloaded Waves from GitHub and see **"Waves.app is damaged and can't be opened"**:

**Method 1 - Right-Click to Open (Recommended):**
1. Drag Waves.app to your Applications folder
2. **Right-click** (or Control-click) on Waves.app
3. Select **"Open"** from the menu
4. Click **"Open"** in the security dialog
5. Waves will now open normally in the future

**Method 2 - Terminal Command:**
```bash
# After dragging to Applications folder
sudo xattr -cr /Applications/Waves.app
codesign --force --deep --sign - /Applications/Waves.app
```

**Method 3 - System Settings:**
1. Try to open Waves normally (it will fail)
2. Go to **System Settings > Privacy & Security**
3. Scroll down and click **"Open Anyway"** next to the Waves message
4. Try opening Waves again and click **"Open"**

This is a one-time step required because Waves is not notarized with Apple (which requires a paid developer account).

---

### Quick Install

Run the installation script from the project directory:

```bash
./install-macos.sh
```

This will:
- Build Waves in release mode
- Create a proper macOS application bundle at `/Applications/Waves.app`
- Convert `waves_logo.png` to a proper macOS icon (ICNS format)
- Register file associations for audio files
- Make Waves available in Spotlight and Launchpad

**Note:** The script automatically converts the `waves_logo.png` file into all required icon sizes and creates a proper ICNS icon file for macOS.

## Usage

### Launching Waves

**From Spotlight:**
- Press `Cmd+Space`
- Type "Waves"
- Press Enter

**From Applications:**
- Open Finder
- Go to Applications
- Double-click Waves

**From Command Line:**
```bash
# Open in default directory (~/Music)
waves

# Open specific folder
waves ~/Music/MyAlbum

# Open and play specific file
waves ~/Music/song.mp3
```

### Opening Files from Finder

**Right-click any audio file:**
1. Right-click on an audio file (.mp3, .wav, .flac, .ogg, .m4a)
2. Select "Open With" → "Waves"
3. Waves will open, navigate to the file's folder, and start playing it

**Open folders:**
1. Right-click on any folder
2. Select "Open With" → "Waves"
3. Waves will open at that folder location

### Setting Waves as Default Player (Optional)

To make Waves the default player for audio files:
1. Right-click on an audio file
2. Select "Get Info"
3. Under "Open with:", select Waves
4. Click "Change All..."

## Supported Formats

- MP3 (.mp3)
- WAV (.wav)
- FLAC (.flac)
- OGG (.ogg)
- M4A (.m4a)

## Keyboard Shortcuts

See the in-app status bar for all available shortcuts, including:
- `h/j/k/l` - Navigate (vim-style)
- `Space` - Pause/Resume
- `←/→` - Previous/Next track
- `↑/↓` - Volume control
- `y/x/p` - Copy/Cut/Paste files
- `f` - Toggle favorite
- `Tab` - Switch views

## Uninstalling

To remove Waves:
```bash
rm -rf /Applications/Waves.app
```

## macOS Troubleshooting

**Waves doesn't appear in "Open With" menu:**
- Run the install script again to re-register file associations
- Log out and log back in

**Can't launch Waves:**
- Make sure you ran the installation script
- Check that `/Applications/Waves.app` exists
- Try running from terminal: `open /Applications/Waves.app`

---

## Linux Installation

### Quick Install

Run the installation script from the project directory:

```bash
chmod +x install-linux.sh
./install-linux.sh
```

This will:
- Build Waves in release mode
- Install the binary to `~/.local/bin/waves`
- Create a desktop entry for application launchers
- Register file associations for audio files

### Post-Installation

Make sure `~/.local/bin` is in your PATH. Add this to your `~/.bashrc` or `~/.zshrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Then reload your shell configuration:
```bash
source ~/.bashrc  # or source ~/.zshrc
```

### Usage

**From Command Line:**
```bash
# Open in default directory (~/Music or system audio directory)
waves

# Open specific folder
waves ~/Music/MyAlbum

# Open and play specific file
waves ~/Music/song.mp3
```

**From Application Launcher:**
- Search for "WAVES" in your application menu
- Click to launch

**Opening Files from File Manager:**
- Right-click any audio file (.mp3, .wav, .flac, .ogg, .m4a)
- Select "Open With" → "WAVES"

### Linux Uninstalling

```bash
rm ~/.local/bin/waves
rm ~/.local/share/applications/waves.desktop
```

---

## Windows Installation

### Quick Install

Run the installation script from PowerShell (as Administrator recommended):

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
.\install-windows.ps1
```

This will:
- Build Waves in release mode
- Install the executable to `%LOCALAPPDATA%\Programs\WAVES`
- Add WAVES to your PATH
- Register file associations for audio files

### Post-Installation

**Restart your terminal** for PATH changes to take effect.

### Usage

**From Command Line:**
```powershell
# Open in default directory (Music library or current directory)
waves

# Open specific folder
waves C:\Users\YourName\Music\MyAlbum

# Open and play specific file
waves C:\Users\YourName\Music\song.mp3
```

**From File Explorer:**
- Double-click any audio file associated with WAVES
- Or right-click → "Open with" → "WAVES"

### Windows Uninstalling

Run in PowerShell:
```powershell
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\Programs\WAVES"
```

Then remove from PATH manually through System Environment Variables.

---

## Cross-Platform Configuration

WAVES uses a configuration file located at:
- **macOS/Linux:** `~/.waves.yml`
- **Windows:** `%USERPROFILE%\.waves.yml`

### Available Options

```yaml
# Enable/disable spectrum visualization
animation: true

# Sidebar position: "left" or "right"
sidebar_position: left

# Show/hide window decorations (title bar)
decorations: true

# Window corner rounding in pixels
window_corner_radius: 0.0

# Default folder on startup (uses ~ expansion)
default_folder: "~/Music"

# Show/hide keyboard shortcuts status bar
show_status_bar: true

# Primary UI accent color (hex format)
primary_color: "#9664FF"

# Window opacity percentage (0-100)
window_opacity: 100.0

# Custom font path (optional, uses ~ expansion)
custom_font: "~/path/to/your/font.ttf"
```

### Platform-Specific Examples

**macOS:**
```yaml
custom_font: "~/Library/Fonts/GohuFont14NerdFontMono-Regular.ttf"
default_folder: "~/Music"
```

**Linux:**
```yaml
custom_font: "~/.local/share/fonts/MyFont.ttf"
default_folder: "~/Music"
```

**Windows:**
```yaml
custom_font: "C:/Windows/Fonts/consola.ttf"
default_folder: "C:/Users/YourName/Music"
```

### Favorites

Favorites are stored in:
- **macOS/Linux:** `~/.waves/favorites.yml`
- **Windows:** `%USERPROFILE%\.waves\favorites.yml`

---

## General Keyboard Shortcuts

These shortcuts work on all platforms:

- `h/j/k/l` - Navigate (vim-style: left/down/up/right)
- `Enter` / `l` - Select directory or play file
- `Space` - Pause/Resume playback
- `←` / `→` - Previous/Next track
- `↑` / `↓` - Increase/Decrease volume
- `y` - Copy (yank) file/folder
- `x` - Cut file/folder
- `p` - Paste file/folder
- `r` - Rename selected item
- `d` - Delete selected item (with confirmation)
- `n` - Create new folder
- `f` - Toggle favorite
- `m` - Open metadata editor (for audio files)
- `Ctrl+F` / `/` - Open search
- `Tab` - Switch between views (Browser/Favorites/Settings)
- `Escape` - Cancel operation
