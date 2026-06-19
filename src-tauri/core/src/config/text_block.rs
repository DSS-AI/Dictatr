use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A reusable text snippet managed by the user in the Prompt-Manager. Grouped
/// by a free-form `category` (the picker renders one tab per distinct category).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextBlock {
    pub id: Uuid,
    pub category: String,
    pub title: String,
    pub content: String,
}
