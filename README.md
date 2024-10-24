# <img src="https://raw.githubusercontent.com/AlexKnauth/socd-cross/refs/heads/cross/app-icon.png" alt="socd-cross" height = "42" align="top"> socd-cross
A utility that allows binding keyboard buttons with SOCD Cleaning. Keybinds are easily customizeable within the app's UI.

Built specifically with Hollow Knight speedruns in mind.

Works on Windows, Mac, and Linux.

## How to Install

Download the latest release for your operating system from:
https://github.com/AlexKnauth/socd-cross/releases/latest

And extract the executable from the zip file.

Troubleshooting:
- On Mac, if you get an error message saying `socd-cross.app is damaged and can't be opened. You should move it to the Trash.`, you can fix it with the command: `xattr -d com.apple.quarantine socd-cross.app`
- On Linux, if you get an error message about `appindicator`, make sure to install the Tauri Linux Prerequisites with one of the commands shown here: https://tauri.app/start/prerequisites/#linux

## How to Compile from Source

Install Tauri CLI

```bash
npm install --save-dev @tauri-apps/cli
```

Install Tauri JavaScript Library

```bash
npm install @tauri-apps/api
```

Start the development server

```bash
npm install
npm run tauri dev
```
