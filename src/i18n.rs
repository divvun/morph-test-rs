use std::collections::HashMap;
use std::sync::OnceLock;

/// Global localization state
static LOCALIZER: OnceLock<Localizer> = OnceLock::new();

/// Initialize the global localizer
pub fn init() {
    let localizer = Localizer::new();
    LOCALIZER
        .set(localizer)
        .expect("Failed to initialize localizer");
}

/// Get a localized message by key
pub fn t(key: &str) -> String {
    LOCALIZER.get().unwrap().get(key)
}

/// Get a localized message by key with arguments (simplified)
pub fn t_with_args(key: &str, args: &[(&str, &dyn std::fmt::Display)]) -> String {
    LOCALIZER.get().unwrap().get_with_args(key, args)
}

/// Localization manager
#[derive(Debug)]
pub struct Localizer {
    messages: HashMap<String, String>,
}

impl Localizer {
    fn new() -> Self {
        let current_language = Self::detect_language();
        let messages = Self::load_messages(&current_language);

        Self { messages }
    }

    fn detect_language() -> String {
        // Always use nn-Runr locale (Norwegian Nynorsk written with runes)
        "nn-Runr".to_string()
    }

    fn load_messages(language: &str) -> HashMap<String, String> {
        let mut messages = HashMap::new();

        // Always load the nn-Runr locale file
        let content = include_str!("../locales/nn-Runr.ftl");

        // Parse simple key = value format
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once(" = ") {
                messages.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        messages
    }

    fn get(&self, key: &str) -> String {
        self.messages.get(key).cloned().unwrap_or_else(|| {
            eprintln!("Missing translation key: {key}");
            format!("MISSING: {key}")
        })
    }

    fn get_with_args(&self, key: &str, args: &[(&str, &dyn std::fmt::Display)]) -> String {
        let mut message = self.get(key);

        // Simple string replacement for {$var} patterns
        for (var_name, value) in args {
            let placeholder = format!("{{${var_name}}}");
            message = message.replace(&placeholder, &format!("{value}"));
        }

        message
    }
}

// Convenience macros for common usage patterns
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::i18n::t($key)
    };
}

#[macro_export]
macro_rules! t_args {
    ($key:expr, $($name:expr => $value:expr),*) => {{
        let args: &[(&str, &dyn std::fmt::Display)] = &[
            $(
                ($name, &$value),
            )*
        ];
        $crate::i18n::t_with_args($key, args)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_localization() {
        let localizer = Localizer::new();

        // Test that we can get a basic message
        let message = localizer.get("cli-about");
        assert!(!message.is_empty());
    }

    #[test]
    fn test_language_detection() {
        // Test that language detection doesn't panic
        let _lang = Localizer::detect_language();
    }
}
