use crate::error::{AppError, Result};
use enigo::{Enigo, Keyboard, Settings};

pub struct TextInjector;

impl TextInjector {
    pub fn inject(text: &str) -> Result<()> {
        let mut enigo = Enigo::new(&Settings::default())
            .map_err(|e| AppError::Inject(e.to_string()))?;
        enigo.text(text).map_err(|e| AppError::Inject(e.to_string()))?;
        Ok(())
    }

    pub fn clipboard_fallback(text: &str) -> Result<()> {
        let mut cb = arboard::Clipboard::new().map_err(|e| AppError::Inject(e.to_string()))?;
        cb.set_text(text.to_string()).map_err(|e| AppError::Inject(e.to_string()))?;
        Ok(())
    }
}
