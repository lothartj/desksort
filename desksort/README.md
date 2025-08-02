# DeskSort

A cross-platform desktop application that automatically organizes your desktop files into categorized folders. Built with Tauri (Rust) and vanilla web technologies.

## Features

- 🔍 **Desktop Scanner**: Scans your desktop for files and folders
- 📁 **Automatic Categorization**: Sorts files based on their extensions
- ⚙️ **Custom Configuration**: Map file categories to custom destination folders
- 🧼 **Smart Sorting**: Handles duplicates and creates missing folders automatically
- 💻 **Cross-Platform**: Works on Windows, macOS, and Linux

## Supported File Categories

- Documents (.pdf, .docx, .doc, .txt, .odt, .rtf)
- Spreadsheets (.xls, .xlsx, .csv, .ods)
- Presentations (.pptx, .odp, .key)
- Images (.jpg, .jpeg, .png, .gif, .bmp, .webp, .tiff)
- Videos (.mp4, .mkv, .avi, .mov, .webm, .flv, .wmv)
- Audio (.mp3, .wav, .aac, .ogg, .flac)
- Archives (.zip, .rar, .7z, .tar, .gz, .tar.gz)
- Executables (.exe, .msi, .sh, .bat, .AppImage)
- Code (.js, .py, .rs, .cpp, .java, .html, .css, .json, .ts)
- Folders (any directory)

## Development

### Prerequisites

1. Install [Rust](https://rustup.rs/)
2. Install [Node.js](https://nodejs.org/)
3. Install Tauri CLI:
   ```bash
   cargo install tauri-cli
   ```

### Setup

1. Clone the repository
2. Install dependencies:
   ```bash
   npm install
   ```

### Running in Development Mode

```bash
npm run tauri dev
```

### Building for Production

```bash
npm run tauri build
```

## Configuration

DeskSort stores its configuration in:
- Windows: `%APPDATA%\desksort\settings.json`
- macOS: `~/Library/Application Support/desksort/settings.json`
- Linux: `~/.config/desksort/settings.json`

The configuration file maps file categories to destination folders. On first run, DeskSort creates default mappings in a "Sorted" folder on your desktop.

## License

MIT 