const DEFAULT_SYSTEM: &str = "Du korrigierst diktierten Text. Verändere den Inhalt nicht. \
Korrigiere ausschließlich Rechtschreibung, Grammatik, Zeichensetzung und offensichtlich \
falsche Wort-Erkennungen. Gib ausschließlich den korrigierten Text zurück, ohne Kommentare \
oder Anführungszeichen.";

pub struct PostProcPrompt {
    pub system: String,
    pub user: String,
}

pub fn build_prompt(
    raw_text: &str,
    vocabulary: &[String],
    custom_system: Option<&str>,
) -> PostProcPrompt {
    let base = custom_system.unwrap_or(DEFAULT_SYSTEM);
    let system = if vocabulary.is_empty() {
        base.to_string()
    } else {
        format!(
            "{base}\n\nVerwende folgendes Vokabular korrekt, wenn es vorkommt:\n{}",
            vocabulary.join(", ")
        )
    };
    PostProcPrompt { system, user: raw_text.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_system_used_when_no_custom() {
        let p = build_prompt("hallo welt", &[], None);
        assert!(p.system.starts_with("Du korrigierst"));
        assert_eq!(p.user, "hallo welt");
    }

    #[test]
    fn vocabulary_appended_to_system() {
        let p = build_prompt("x", &vec!["DSS-Siegmund".into(), "Invoice Ninja".into()], None);
        assert!(p.system.contains("DSS-Siegmund, Invoice Ninja"));
    }

    #[test]
    fn custom_system_replaces_default() {
        let p = build_prompt("x", &[], Some("Antworte nur in Großbuchstaben."));
        assert!(p.system.contains("Großbuchstaben"));
        assert!(!p.system.contains("Du korrigierst"));
    }
}
