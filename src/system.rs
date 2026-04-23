use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};

pub struct HotkeyController {
    _manager: GlobalHotKeyManager,
    hotkey: HotKey,
}

impl HotkeyController {
    pub fn register_default() -> Result<Self, String> {
        let manager = GlobalHotKeyManager::new().map_err(|err| err.to_string())?;
        let hotkey = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyP);
        manager.register(hotkey).map_err(|err| err.to_string())?;
        Ok(Self {
            _manager: manager,
            hotkey,
        })
    }

    pub fn was_pressed(&self) -> bool {
        let receiver = GlobalHotKeyEvent::receiver();
        let mut pressed = false;
        while let Ok(event) = receiver.try_recv() {
            if event.id == self.hotkey.id() {
                pressed = true;
            }
        }
        pressed
    }
}

pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    Clipboard::new()
        .map_err(|err| err.to_string())?
        .set_text(text.to_owned())
        .map_err(|err| err.to_string())
}

pub fn paste_from_clipboard() -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|err| err.to_string())?;
    enigo
        .key(Key::Meta, Direction::Press)
        .map_err(|err| err.to_string())?;
    enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|err| err.to_string())?;
    enigo
        .key(Key::Meta, Direction::Release)
        .map_err(|err| err.to_string())
}
