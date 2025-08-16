use std::cell::RefCell;
use yew::prelude::*;

thread_local! {
    static CURRENT_LANGUAGE: RefCell<Option<String>> = RefCell::new(None);
}

#[derive(Clone, Debug, PartialEq)]
pub struct LanguageContext {
    pub lang: String,
    pub dir: TextDirection,
}

/// Text direction for language
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextDirection {
    LTR,
    RTL,
}

impl TextDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LTR => "ltr",
            Self::RTL => "rtl",
        }
    }

    pub fn from_lang(lang: &str) -> Self {
        // RTL languages
        match lang {
            "ar" | "he" | "fa" | "ur" => Self::RTL,
            _ => Self::LTR,
        }
    }
}

impl LanguageContext {
    pub fn new(lang: impl Into<String>) -> Self {
        let lang = lang.into();
        let dir = TextDirection::from_lang(&lang);
        Self { lang, dir }
    }

    pub fn fallback() -> Self {
        Self {
            lang: "en".to_string(),
            dir: TextDirection::LTR,
        }
    }

    /// Set the current language in thread-local storage
    pub fn set_thread_local_lang(lang: &str) {
        CURRENT_LANGUAGE.with(|current| {
            *current.borrow_mut() = Some(lang.to_string());
        });
    }

    /// Get the current language from thread-local storage
    pub fn get_thread_local_lang() -> Option<String> {
        CURRENT_LANGUAGE.with(|current| current.borrow().clone())
    }

    /// Clear the thread-local language
    pub fn clear_thread_local_lang() {
        CURRENT_LANGUAGE.with(|current| {
            *current.borrow_mut() = None;
        });
    }

    /// Get the current language from various sources in priority order:
    /// 1. Thread-local storage
    /// 2. Environment variable YEW_SSG_CURRENT_LANG
    /// 3. Default fallback
    pub fn get_current_lang() -> String {
        // First try thread-local
        if let Some(lang) = Self::get_thread_local_lang() {
            return lang;
        }

        // Then try environment variable
        if let Ok(lang) = std::env::var("YEW_SSG_CURRENT_LANG") {
            return lang;
        }

        // Finally use default
        "en".to_string()
    }
}

/// Properties for the LanguageProvider component
#[derive(Properties, PartialEq)]
pub struct LanguageProviderProps {
    #[prop_or_else(|| "en".to_string())]
    pub lang: String,
    #[prop_or_default]
    pub children: Children,
}

/// Provides language context to child components
#[function_component(LanguageProvider)]
pub fn language_provider(props: &LanguageProviderProps) -> Html {
    let context = LanguageContext::new(&props.lang);

    // Set the thread-local language when provider is created
    LanguageContext::set_thread_local_lang(&props.lang);

    html! {
        <ContextProvider<LanguageContext> context={context}>
            {props.children.clone()}
        </ContextProvider<LanguageContext>>
    }
}

/// Hook to access the current language context
#[hook]
pub fn use_language() -> LanguageContext {
    // First try to get from Yew context
    if let Some(context) = use_context::<LanguageContext>() {
        return context;
    }

    // Fallback to thread-local or environment variable
    let lang = LanguageContext::get_current_lang();
    LanguageContext::new(lang)
}
