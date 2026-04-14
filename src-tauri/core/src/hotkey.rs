use crate::error::{AppError, Result};
use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::collections::HashMap;
use std::str::FromStr;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum HotkeyEvent {
    Pressed(Uuid),
    Released(Uuid),
}

pub struct HotkeyRegistry {
    manager: GlobalHotKeyManager,
    by_id: HashMap<u32, Uuid>,
}

impl HotkeyRegistry {
    pub fn new() -> Result<Self> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| AppError::Config(format!("hotkey manager init failed: {e}")))?;
        Ok(Self { manager, by_id: HashMap::new() })
    }

    pub fn register(&mut self, profile_id: Uuid, combo: &str) -> Result<()> {
        let hk = HotKey::from_str(combo)
            .map_err(|e| AppError::Config(format!("invalid hotkey {combo}: {e}")))?;
        self.manager.register(hk).map_err(|e| AppError::Config(e.to_string()))?;
        self.by_id.insert(hk.id(), profile_id);
        Ok(())
    }

    pub fn clear(&mut self) {
        let _ = self.manager.unregister_all(&[]);
        self.by_id.clear();
    }

    pub fn resolve(&self, hk_id: u32) -> Option<Uuid> {
        self.by_id.get(&hk_id).copied()
    }

    pub fn pump_into(&self, tx: UnboundedSender<HotkeyEvent>) {
        Self::pump(self.by_id.clone(), tx);
    }

    pub fn id_map(&self) -> HashMap<u32, Uuid> {
        self.by_id.clone()
    }

    pub fn pump(by_id: HashMap<u32, Uuid>, tx: UnboundedSender<HotkeyEvent>) {
        let receiver = GlobalHotKeyEvent::receiver();
        while let Ok(event) = receiver.recv() {
            let profile_id = match by_id.get(&event.id).copied() {
                Some(id) => id,
                None => continue,
            };
            let msg = match event.state {
                HotKeyState::Pressed => HotkeyEvent::Pressed(profile_id),
                HotKeyState::Released => HotkeyEvent::Released(profile_id),
            };
            if tx.send(msg).is_err() { break; }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_combo() {
        let hk = HotKey::from_str("Ctrl+Alt+Space");
        assert!(hk.is_ok());
    }

    #[test]
    fn rejects_garbage_combo() {
        let hk = HotKey::from_str("not a hotkey");
        assert!(hk.is_err());
    }
}
