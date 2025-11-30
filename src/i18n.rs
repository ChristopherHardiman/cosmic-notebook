//! Internationalization (i18n) module for Cosmic Notebook
//!
//! Provides localization support using fluent-rs for translations.
//! Supports runtime language switching and follows COSMIC desktop locale settings.

// Simplified i18n for Phase 1
// Full fluent integration will be added in a later phase

/// Initialize localization with system locale
pub fn init() {
    log::debug!("i18n initialization - using default English strings");
}

/// Get the current language code
pub fn current_language() -> String {
    "en".to_string()
}

/// Macro for accessing localized strings (placeholder for Phase 1)
/// Returns the message_id as-is until full i18n is implemented
#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {
        $message_id
    };
    ($message_id:literal, $($arg:tt)*) => {
        $message_id
    };
}

