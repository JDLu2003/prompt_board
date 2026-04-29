use arboard::Clipboard;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
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
            if event.id == self.hotkey.id() && event.state == HotKeyState::Pressed {
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
