use crate::error::{AppError, Result};
use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum HotkeyEvent {
    Pressed(Uuid),
    Released(Uuid),
}

/// Thread-safe map of global-hotkey IDs → profile IDs. Shared between the
/// hotkey-owner thread (writes on reload) and the pump thread (reads on event).
pub type SharedIdMap = Arc<Mutex<HashMap<u32, Uuid>>>;

pub struct HotkeyRegistry {
    manager: GlobalHotKeyManager,
    by_id: HashMap<u32, Uuid>,
    /// Registered HotKey objects, so `clear()` can actually unregister them.
    hotkeys: Vec<HotKey>,
    /// Multimedia / launch keys registered via the low-level hook
    /// (vk-code → profile id). Populated by `register()` when the combo
    /// is a known multimedia key name.
    ll_keys: HashMap<u32, Uuid>,
}

impl HotkeyRegistry {
    pub fn new() -> Result<Self> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| AppError::Config(format!("hotkey manager init failed: {e}")))?;
        Ok(Self {
            manager,
            by_id: HashMap::new(),
            hotkeys: Vec::new(),
            ll_keys: HashMap::new(),
        })
    }

    pub fn register(&mut self, profile_id: Uuid, combo: &str) -> Result<()> {
        // Multimedia / launch keys go through the low-level hook, which is
        // the only reliable way to intercept them on Windows.
        if let Some(vk) = crate::hotkey_ll::parse_vk(combo) {
            self.ll_keys.insert(vk, profile_id);
            return Ok(());
        }
        let hk = HotKey::from_str(combo)
            .map_err(|e| AppError::Config(format!("invalid hotkey {combo}: {e}")))?;
        self.manager.register(hk).map_err(|e| AppError::Config(e.to_string()))?;
        self.by_id.insert(hk.id(), profile_id);
        self.hotkeys.push(hk);
        Ok(())
    }

    /// Multimedia-key vk-code → profile id map for `hotkey_ll::start` /
    /// `hotkey_ll::update_mapping`.
    pub fn ll_keys(&self) -> HashMap<u32, Uuid> {
        self.ll_keys.clone()
    }

    pub fn clear(&mut self) {
        if !self.hotkeys.is_empty() {
            let _ = self.manager.unregister_all(&self.hotkeys);
        }
        self.hotkeys.clear();
        self.by_id.clear();
        self.ll_keys.clear();
    }

    pub fn resolve(&self, hk_id: u32) -> Option<Uuid> {
        self.by_id.get(&hk_id).copied()
    }

    pub fn id_map(&self) -> HashMap<u32, Uuid> {
        self.by_id.clone()
    }

    /// Pump events from the global-hotkey channel, looking up the profile id
    /// from a shared map that can be updated while the pump runs.
    pub fn pump_shared(map: SharedIdMap, tx: UnboundedSender<HotkeyEvent>) {
        let receiver = GlobalHotKeyEvent::receiver();
        while let Ok(event) = receiver.recv() {
            let profile_id = map.lock().get(&event.id).copied();
            let Some(profile_id) = profile_id else { continue };
            let msg = match event.state {
                HotKeyState::Pressed => HotkeyEvent::Pressed(profile_id),
                HotKeyState::Released => HotkeyEvent::Released(profile_id),
            };
            if tx.send(msg).is_err() {
                break;
            }
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
