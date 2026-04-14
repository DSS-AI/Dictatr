use crate::error::{AppError, Result};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

pub struct TextInjector;

impl TextInjector {
    /// Inject the text into the focused field via clipboard paste.
    ///
    /// `enigo.text()` (SendInput with unicode characters) regularly drops
    /// the first and/or last character when the target app is briefly
    /// unfocused or buffers keystrokes, and mishandles some unicode
    /// combinations. Putting the text on the clipboard and sending Ctrl+V
    /// is the robust path every major dictation tool uses.
    pub fn inject(text: &str) -> Result<()> {
        // Preserve whatever the user had on the clipboard.
        let prev = arboard::Clipboard::new()
            .ok()
            .and_then(|mut cb| cb.get_text().ok());

        {
            let mut cb = arboard::Clipboard::new()
                .map_err(|e| AppError::Inject(e.to_string()))?;
            cb.set_text(text.to_string())
                .map_err(|e| AppError::Inject(e.to_string()))?;
        }

        // Short delay so the clipboard set actually lands before the paste
        // chord fires — also covers the "target app not yet accepting input"
        // race that causes lost leading characters.
        std::thread::sleep(std::time::Duration::from_millis(60));

        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| AppError::Inject(e.to_string()))?;
        enigo
            .key(Key::Control, Direction::Press)
            .map_err(|e| AppError::Inject(e.to_string()))?;
        enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| AppError::Inject(e.to_string()))?;
        enigo
            .key(Key::Control, Direction::Release)
            .map_err(|e| AppError::Inject(e.to_string()))?;

        // Give the paste a moment to complete before restoring the
        // clipboard, otherwise the target app might see the restored text.
        if let Some(prev_text) = prev {
            std::thread::sleep(std::time::Duration::from_millis(250));
            if let Ok(mut cb) = arboard::Clipboard::new() {
                let _ = cb.set_text(prev_text);
            }
        }
        Ok(())
    }

    pub fn clipboard_fallback(text: &str) -> Result<()> {
        let mut cb = arboard::Clipboard::new().map_err(|e| AppError::Inject(e.to_string()))?;
        cb.set_text(text.to_string()).map_err(|e| AppError::Inject(e.to_string()))?;
        Ok(())
    }
}
