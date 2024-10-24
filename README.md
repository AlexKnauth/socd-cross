# <img src="https://raw.githubusercontent.com/AlexKnauth/socd-cross/refs/heads/cross/app-icon.png" alt="socd-cross" height = "42" align="top"> socd-cross
A utility that allows binding keyboard buttons with SOCD Cleaning. Keybinds are easily customizeable within the app's UI.

Built specifically with Hollow Knight speedruns in mind.

Works on Windows and Mac. I have not gotten it to work on Linux yet, but theoretically it could be made to.

## How to Install

Download the latest release for your operating system from:
https://github.com/AlexKnauth/socd-cross/releases/latest

And extract the executable from the zip file.

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
