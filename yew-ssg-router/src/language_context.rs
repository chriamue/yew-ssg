use yew::prelude::*;

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

    html! {
        <ContextProvider<LanguageContext> context={context}>
            {props.children.clone()}
        </ContextProvider<LanguageContext>>
    }
}

/// Hook to access the current language context
#[hook]
pub fn use_language() -> LanguageContext {
    use_context::<LanguageContext>().unwrap_or_else(LanguageContext::fallback)
}
