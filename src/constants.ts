export const WEB_TO_RDEV_KEYMAP: Record<string, string> = {
  'Backspace': 'Backspace',
  'Tab': 'Tab',
  'Enter': 'Return',
  'Shift': 'ShiftLeft', // both Shift and ShiftLeft map to ShiftLeft
  'Control': 'ControlLeft', // both Control and ControlLeft map to ControlLeft
  'Alt': 'Alt', // Alt, AltLeft, and AltRight all map to Alt
  'Pause': 'Pause',
  'CapsLock': 'CapsLock',
  'Escape': 'Escape',
  'Space': 'Space',
  'PageUp': 'PageUp',
  'PageDown': 'PageDown',
  'End': 'End',
  'Home': 'Home',
  'ArrowLeft': 'LeftArrow',
  'ArrowUp': 'UpArrow',
  'ArrowRight': 'RightArrow',
  'ArrowDown': 'DownArrow',
  'PrintScreen': 'PrintScreen',
  'Insert': 'Insert',
  'Delete': 'Delete',
  'Digit0': 'Num0', // TODO: is this Num thing right? or is it Numpad? but then what's Kp for?
  'Digit1': 'Num1',
  'Digit2': 'Num2',
  'Digit3': 'Num3',
  'Digit4': 'Num4',
  'Digit5': 'Num5',
  'Digit6': 'Num6',
  'Digit7': 'Num7',
  'Digit8': 'Num8',
  'Digit9': 'Num9',
  'KeyA': 'KeyA',
  'KeyB': 'KeyB',
  'KeyC': 'KeyC',
  'KeyD': 'KeyD',
  'KeyE': 'KeyE',
  'KeyF': 'KeyF',
  'KeyG': 'KeyG',
  'KeyH': 'KeyH',
  'KeyI': 'KeyI',
  'KeyJ': 'KeyJ',
  'KeyK': 'KeyK',
  'KeyL': 'KeyL',
  'KeyM': 'KeyM',
  'KeyN': 'KeyN',
  'KeyO': 'KeyO',
  'KeyP': 'KeyP',
  'KeyQ': 'KeyQ',
  'KeyR': 'KeyR',
  'KeyS': 'KeyS',
  'KeyT': 'KeyT',
  'KeyU': 'KeyU',
  'KeyV': 'KeyV',
  'KeyW': 'KeyW',
  'KeyX': 'KeyX',
  'KeyY': 'KeyY',
  'KeyZ': 'KeyZ',
  'MetaLeft': 'MetaLeft', // Windows Key
  'MetaRight': 'MetaRight', // Windows Key
  // 'ContextMenu': 0x5D,
  'Numpad0': 'Kp0',
  'Numpad1': 'Kp1',
  'Numpad2': 'Kp2',
  'Numpad3': 'Kp3',
  'Numpad4': 'Kp4',
  'Numpad5': 'Kp5',
  'Numpad6': 'Kp6',
  'Numpad7': 'Kp7',
  'Numpad8': 'Kp8',
  'Numpad9': 'Kp9',
  'Multiply': 'KpMultiply',
  'Add': 'KpPlus',
  'Subtract': 'KpMinus',
  'Decimal': 'KpDelete',
  'Divide': 'KpDivide',
  'F1': 'F1',
  'F2': 'F2',
  'F3': 'F3',
  'F4': 'F4',
  'F5': 'F5',
  'F6': 'F6',
  'F7': 'F7',
  'F8': 'F8',
  'F9': 'F9',
  'F10': 'F10',
  'F11': 'F11',
  'F12': 'F12',
  'NumLock': 'NumLock',
  'ScrollLock': 'ScrollLock',
  'ShiftLeft': 'ShiftLeft', // both Shift and ShiftLeft map to ShiftLeft
  'ShiftRight': 'ShiftRight',
  'ControlLeft': 'ControlLeft', // both Control and ControlLeft map to ControlLeft
  'ControlRight': 'ControlRight',
  'AltLeft': 'Alt', // Alt, AltLeft, and AltRight all map to Alt
  'AltRight': 'Alt', // Alt, AltLeft, and AltRight all map to Alt
  'Semicolon': 'SemiColon',
  'Equal': 'Equal',
  'Comma': 'Comma',
  'Minus': 'Minus',
  'Period': 'Dot',
  'Slash': 'Slash',
  'Backquote': 'BackQuote',
  'BracketLeft': 'LeftBracket',
  'Backslash': 'BackSlash',
  'BracketRight': 'RightBracket',
  'Quote': 'Quote',
  // Add more mappings as needed
}

export const MOD_KEYS: Set<string> = new Set([
  'Alt', // Alt
  'ControlLeft', // Control
  'ShiftLeft', // Shift
  'MetaLeft', // Windows
]);
