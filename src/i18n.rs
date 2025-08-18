use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use unic_locale::LanguageIdentifier;
use isolang::Language;

/// Global localization state
static LOCALIZER: OnceLock<Mutex<Localizer>> = OnceLock::new();

/// Initialize the global localizer
pub fn init() {
    let localizer = Localizer::new();
    LOCALIZER
        .set(Mutex::new(localizer))
        .expect("Failed to initialize localizer");
}

/// Get a localized message by key
pub fn t(key: &str) -> String {
    LOCALIZER
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .get(key)
}

/// Get a localized message by key with arguments
pub fn t_with_args(key: &str, args: &[(&str, &dyn std::fmt::Display)]) -> String {
    LOCALIZER
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .get_with_args(key, args)
}

/// Localization manager with BCP-47 support and graceful fallbacks
#[derive(Debug)]
pub struct Localizer {
    messages: HashMap<String, String>,
    available_locales: Vec<LanguageIdentifier>,
    current_locale: LanguageIdentifier,
}

impl Localizer {
    fn new() -> Self {
        let mut localizer = Self {
            messages: HashMap::new(),
            available_locales: Vec::new(),
            current_locale: "en".parse().unwrap(),
        };
        
        // Discover available locales from the locales directory
        localizer.discover_available_locales();
        
        // Detect the best locale to use
        let selected_locale = localizer.select_best_locale();
        
        // Load messages for the selected locale
        localizer.load_locale(&selected_locale);
        
        localizer
    }

    /// Discover all available locales by scanning the locales directory
    fn discover_available_locales(&mut self) {
        let locales_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("locales");
        
        if let Ok(entries) = fs::read_dir(&locales_dir) {
            for entry in entries.flatten() {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".ftl") {
                        let locale_str = filename.strip_suffix(".ftl").unwrap();
                        
                        // Try to parse as a valid BCP-47 locale
                        if let Ok(locale_id) = locale_str.parse::<LanguageIdentifier>() {
                            self.available_locales.push(locale_id);
                        }
                    }
                }
            }
        }
        
        // Ensure English is always available as fallback
        let en: LanguageIdentifier = "en".parse().unwrap();
        if !self.available_locales.contains(&en) {
            self.available_locales.push(en);
        }
        
        // Sort by specificity (more specific first)
        self.available_locales.sort_by(|a, b| {
            let a_parts = format!("{}", a).matches('-').count();
            let b_parts = format!("{}", b).matches('-').count();
            b_parts.cmp(&a_parts)
        });
    }

    /// Select the best available locale based on system preferences
    fn select_best_locale(&self) -> LanguageIdentifier {
        // Get user preferences from system locale and environment
        let user_locales = self.get_user_locale_preferences();
        
        // Find the best match using fallback logic
        for user_locale in &user_locales {
            if let Some(best_match) = self.find_best_match(user_locale) {
                return best_match;
            }
        }
        // Ultimate fallback to English
        "en".parse().unwrap()
    }

    /// Get user locale preferences from environment variables
    fn get_user_locale_preferences(&self) -> Vec<LanguageIdentifier> {
        let mut preferences = Vec::new();
        
        // Check standard environment variables first (higher priority)
        let env_vars = ["LC_ALL", "LC_MESSAGES", "LANG"];
        for var in &env_vars {
            if let Ok(value) = std::env::var(var) {
                // Parse locale, removing encoding (e.g., "nn-Runr.UTF-8" -> "nn-Runr")
                let locale_str = value.split('.').next().unwrap_or(&value).replace('_', "-");
                if let Ok(locale_id) = locale_str.parse::<LanguageIdentifier>() {
                    if !preferences.contains(&locale_id) {
                        preferences.push(locale_id);
                    }
                }
            }
        }
        
        // Try system locale as fallback
        if let Some(system_locale) = sys_locale::get_locale() {
            if let Ok(locale_id) = system_locale.replace('_', "-").parse::<LanguageIdentifier>() {
                if !preferences.contains(&locale_id) {
                    preferences.push(locale_id);
                }
            }
        }
        
        // Add English as final fallback
        let en: LanguageIdentifier = "en".parse().unwrap();
        if !preferences.contains(&en) {
            preferences.push(en);
        }
        
        preferences
    }

    /// Normalize language codes using ISO 639 standards
    /// Converts three-letter codes to two-letter equivalents when available
    fn normalize_language_code(&self, lang_code: &str) -> String {
        // Try to parse the language code using isolang
        if Language::from_639_1(lang_code).is_some() {
            // It's already a two-letter code, return as-is
            return lang_code.to_string();
        }
        
        if let Some(language) = Language::from_639_3(lang_code) {
            // It's a three-letter code, convert to two-letter if available
            if let Some(two_letter) = language.to_639_1() {
                return two_letter.to_string();
            }
        }
        
        // If we can't normalize it, return the original code
        lang_code.to_string()
    }

    /// Find the best matching locale with graceful fallbacks
    fn find_best_match(&self, requested: &LanguageIdentifier) -> Option<LanguageIdentifier> {
        // Try exact match first
        if self.available_locales.contains(requested) {
            return Some(requested.clone());
        }
        
        // Build fallback chain
        let mut fallback_chain = Vec::new();
        
        // Start with the original request
        fallback_chain.push(requested.clone());
        
        // Try removing variant if present
        if requested.variants().next().is_some() {
            let without_variant = LanguageIdentifier::from_parts(
                requested.language,
                requested.script,
                requested.region,
                &[],
            );
            fallback_chain.push(without_variant);
        }
        
        // Try removing region if present
        if requested.region.is_some() {
            let without_region = LanguageIdentifier::from_parts(
                requested.language,
                requested.script,
                None,
                &[],
            );
            fallback_chain.push(without_region);
        }
        
        // Try normalized language code with same script/region
        let normalized_lang = self.normalize_language_code(&requested.language.to_string());
        if normalized_lang != requested.language.to_string() {
            // Try normalized language with script and region
            if let Ok(norm_lang) = normalized_lang.parse() {
                if requested.script.is_some() || requested.region.is_some() {
                    let normalized_full = LanguageIdentifier::from_parts(
                        norm_lang,
                        requested.script,
                        requested.region,
                        &[],
                    );
                    fallback_chain.push(normalized_full);
                }
                
                // Try normalized language with script only (no region)
                if requested.script.is_some() {
                    let normalized_script = LanguageIdentifier::from_parts(
                        norm_lang,
                        requested.script,
                        None,
                        &[],
                    );
                    fallback_chain.push(normalized_script);
                }
                
                // Try normalized language only
                let normalized_lang_only = LanguageIdentifier::from_parts(
                    norm_lang,
                    None,
                    None,
                    &[],
                );
                fallback_chain.push(normalized_lang_only);
            }
        }
        
        // Try just language if script was present
        if requested.script.is_some() || requested.region.is_some() {
            let lang_only = LanguageIdentifier::from_parts(
                requested.language,
                None,
                None,
                &[],
            );
            fallback_chain.push(lang_only);
        }
        
        // Check each fallback
        for fallback in fallback_chain {
            if self.available_locales.contains(&fallback) {
                return Some(fallback);
            }
        }
        
        None
    }

    /// Load messages for the specified locale
    fn load_locale(&mut self, locale: &LanguageIdentifier) {
        let locale_str = locale.to_string();
        let locales_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("locales");
        let file_path = locales_dir.join(format!("{}.ftl", locale_str));
        
        if let Ok(content) = fs::read_to_string(&file_path) {
            self.messages = self.parse_fluent_content(&content);
            self.current_locale = locale.clone();
            return;
        }
        
        // Fallback to English if requested locale failed to load
        if locale_str != "en" {
            let en_locale: LanguageIdentifier = "en".parse().unwrap();
            self.load_locale(&en_locale);
        }
    }

    /// Parse Fluent (.ftl) content into a simple key-value map
    fn parse_fluent_content(&self, content: &str) -> HashMap<String, String> {
        let mut messages = HashMap::new();
        
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

    /// Get a localized message by key
    fn get(&self, key: &str) -> String {
        self.messages.get(key).cloned().unwrap_or_else(|| {
            eprintln!("Missing translation key: {}", key);
            format!("MISSING: {}", key)
        })
    }

    /// Get a localized message with arguments
    fn get_with_args(&self, key: &str, args: &[(&str, &dyn std::fmt::Display)]) -> String {
        let mut message = self.get(key);
        
        // Simple string replacement for {$var} patterns
        for (var_name, value) in args {
            let placeholder = format!("{{${}}}", var_name);
            message = message.replace(&placeholder, &format!("{}", value));
        }
        
        message
    }

    /// Get current locale for debugging
    #[allow(dead_code)]
    pub fn current_locale(&self) -> &LanguageIdentifier {
        &self.current_locale
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
    fn test_locale_discovery() {
        let mut localizer = Localizer {
            messages: HashMap::new(),
            available_locales: Vec::new(),
            current_locale: "en".parse().unwrap(),
        };
        
        localizer.discover_available_locales();
        assert!(!localizer.available_locales.is_empty());
        
        // English should always be available
        let en: LanguageIdentifier = "en".parse().unwrap();
        assert!(localizer.available_locales.contains(&en));
    }

    #[test]
    fn test_fallback_logic() {
        let localizer = Localizer {
            messages: HashMap::new(),
            current_locale: "en".parse().unwrap(),
            available_locales: vec![
                "en".parse().unwrap(),
                "nn".parse().unwrap(),
                "nn-Runr".parse().unwrap(),
            ],
        };

        // Test exact match
        let requested: LanguageIdentifier = "nn-Runr".parse().unwrap();
        assert_eq!(localizer.find_best_match(&requested), Some(requested));

        // Test fallback from region to script
        let with_region: LanguageIdentifier = "nn-Runr-NO".parse().unwrap();
        let expected: LanguageIdentifier = "nn-Runr".parse().unwrap();
        assert_eq!(localizer.find_best_match(&with_region), Some(expected));

        // Test fallback to language only
        let script_only: LanguageIdentifier = "nn-Latn".parse().unwrap();
        let expected_lang: LanguageIdentifier = "nn".parse().unwrap();
        assert_eq!(localizer.find_best_match(&script_only), Some(expected_lang));

        // Test three-letter language code normalization with script
        let nno_runr: LanguageIdentifier = "nno-Runr".parse().unwrap();
        let expected_nn_runr: LanguageIdentifier = "nn-Runr".parse().unwrap();
        assert_eq!(localizer.find_best_match(&nno_runr), Some(expected_nn_runr));

        // Test three-letter language code normalization without script
        let nno_only: LanguageIdentifier = "nno".parse().unwrap();
        let expected_nn: LanguageIdentifier = "nn".parse().unwrap();
        assert_eq!(localizer.find_best_match(&nno_only), Some(expected_nn));
    }

    #[test]
    fn test_locale_parsing() {
        // Test various locale formats
        let test_cases = vec![
            ("en", true),
            ("nn", true),
            ("nn-Runr", true),
            ("nn-Runr-NO", true),
            ("123-invalid", false),  // Invalid language code
        ];

        for (locale_str, should_parse) in test_cases {
            let result = locale_str.parse::<LanguageIdentifier>();
            assert_eq!(result.is_ok(), should_parse, "Failed parsing: {}", locale_str);
        }
    }

    #[test]
    fn test_iso639_normalization() {
        let localizer = Localizer {
            messages: HashMap::new(),
            current_locale: "en".parse().unwrap(),
            available_locales: Vec::new(),
        };

        // Test ISO 639 three-letter to two-letter normalization
        assert_eq!(localizer.normalize_language_code("nno"), "nn");
        assert_eq!(localizer.normalize_language_code("nor"), "no");
        assert_eq!(localizer.normalize_language_code("deu"), "de");
        assert_eq!(localizer.normalize_language_code("fra"), "fr");
        
        // Test that two-letter codes are returned as-is
        assert_eq!(localizer.normalize_language_code("en"), "en");
        assert_eq!(localizer.normalize_language_code("nn"), "nn");
        assert_eq!(localizer.normalize_language_code("de"), "de");
        
        // Test unknown codes
        assert_eq!(localizer.normalize_language_code("xyz"), "xyz");
    }
}