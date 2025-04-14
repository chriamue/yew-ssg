use once_cell::sync::Lazy;
use std::collections::HashMap;

pub static TRANSLATIONS: Lazy<HashMap<&'static str, HashMap<&'static str, &'static str>>> =
    Lazy::new(|| {
        let mut translations = HashMap::new();

        // English translations
        let mut en = HashMap::new();
        en.insert("home", "Home");
        en.insert("about", "About");
        en.insert("readme", "ReadMe");
        en.insert("yew_ssg_crate", "Yew SSG Crate");
        en.insert("yew_ssg_router_crate", "Yew SSG Router Crate");
        en.insert("not_found", "Not Found");
        en.insert("welcome_to_home", "Welcome to Home Page");
        en.insert("simple_example", "This is a simple example using yew-ssg");
        en.insert("about_page", "About Page");
        en.insert(
            "about_description",
            "This is the about page of our example application",
        );
        en.insert("page_not_found", "404 - Page Not Found");
        en.insert(
            "not_found_message",
            "We couldn't find what you were looking for.",
        );
        en.insert("back_to_home", "Back to Home");
        en.insert("language", "Language");

        // German translations
        let mut de = HashMap::new();
        de.insert("home", "Startseite");
        de.insert("about", "Über uns");
        de.insert("readme", "Liesmich");
        de.insert("yew_ssg_crate", "Yew SSG Paket");
        de.insert("yew_ssg_router_crate", "Yew SSG Router Paket");
        de.insert("not_found", "Nicht gefunden");
        de.insert("welcome_to_home", "Willkommen auf der Startseite");
        de.insert(
            "simple_example",
            "Dies ist ein einfaches Beispiel mit yew-ssg",
        );
        de.insert("about_page", "Über uns Seite");
        de.insert(
            "about_description",
            "Dies ist die Über uns Seite unserer Beispielanwendung",
        );
        de.insert("page_not_found", "404 - Seite nicht gefunden");
        de.insert(
            "not_found_message",
            "Wir konnten nicht finden, wonach Sie gesucht haben.",
        );
        de.insert("back_to_home", "Zurück zur Startseite");
        de.insert("language", "Sprache");

        translations.insert("en", en);
        translations.insert("de", de);

        translations
    });

pub fn t(key: &str, lang: &str) -> String {
    let lang_map = TRANSLATIONS
        .get(lang)
        .unwrap_or_else(|| TRANSLATIONS.get("en").unwrap());

    // Try to get the translation for the key
    lang_map
        .get(key)
        .map(|&v| v.to_string())
        .unwrap_or_else(|| key.to_string())
}
