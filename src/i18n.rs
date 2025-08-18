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
        // Check environment variables in order of preference
        let lang_vars = ["LC_ALL", "LC_MESSAGES", "LANG"];

        for var in &lang_vars {
            if let Ok(value) = std::env::var(var) {
                // Handle full locale strings first (e.g., "nn-Runr", "nn_Runr", "nn-Runr_NO")
                let locale_without_encoding = value.split('.').next().unwrap_or(&value);
                let locale_parts: Vec<&str> = locale_without_encoding.split('_').collect();
                
                // Check for nn-Runr variants
                if let Some(lang_script) = locale_parts.first() {
                    match lang_script.to_lowercase().as_str() {
                        "nn-runr" => return "nn-Runr".to_string(),
                        _ => {}
                    }
                }

                // Parse standard locale format (e.g., "nn_NO.UTF-8" -> "nn")
                let lang_code = locale_parts
                    .first()
                    .unwrap_or(&locale_without_encoding)
                    .split('-')
                    .next()
                    .unwrap_or(locale_without_encoding)
                    .to_lowercase();

                match lang_code.as_str() {
                    "nn" | "nno" => return "nn".to_string(),
                    "nb" | "no" | "nor" => return "nb".to_string(),
                    "en" => return "en".to_string(),
                    _ => continue,
                }
            }
        }

        // Default to English
        "en".to_string()
    }

    fn load_messages(language: &str) -> HashMap<String, String> {
        let mut messages = HashMap::new();

        // Load the appropriate language file content
        let content = match language {
            "nn" => include_str!("../locales/nn.ftl"),
            "nn-Runr" => include_str!("../locales/nn-Runr.ftl"),
            "nb" => include_str!("../locales/nb.ftl"),
            _ => include_str!("../locales/en.ftl"), // Default to English
        };

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
