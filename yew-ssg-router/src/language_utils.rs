use crate::language_context::LanguageContext;

/// Language detection utilities for SSG and runtime
pub struct LanguageUtils;

impl LanguageUtils {
    /// Detect language from various sources in priority order
    pub fn detect_language() -> String {
        LanguageContext::get_current_lang()
    }

    /// Set language for current generation context
    pub fn set_generation_language(lang: &str) {
        LanguageContext::set_thread_local_lang(lang);
        std::env::set_var("YEW_SSG_CURRENT_LANG", lang);
    }

    /// Clear language from current generation context
    pub fn clear_generation_language() {
        LanguageContext::clear_thread_local_lang();
        std::env::remove_var("YEW_SSG_CURRENT_LANG");
    }

    /// Create a scoped language context that automatically cleans up
    pub fn with_language<F, R>(lang: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // Save current state
        let old_thread_local = LanguageContext::get_thread_local_lang();
        let old_env = std::env::var("YEW_SSG_CURRENT_LANG").ok();

        // Set new language
        Self::set_generation_language(lang);

        // Execute the function
        let result = f();

        // Restore old state
        match old_thread_local {
            Some(old_lang) => LanguageContext::set_thread_local_lang(&old_lang),
            None => LanguageContext::clear_thread_local_lang(),
        }

        match old_env {
            Some(old_lang) => std::env::set_var("YEW_SSG_CURRENT_LANG", old_lang),
            None => std::env::remove_var("YEW_SSG_CURRENT_LANG"),
        }

        result
    }
}

/// Macro for executing code with a specific language context
#[macro_export]
macro_rules! with_language {
    ($lang:expr, $block:block) => {
        $crate::language_utils::LanguageUtils::with_language($lang, || $block)
    };
}
