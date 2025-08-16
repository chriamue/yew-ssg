#![cfg(test)]

use crate::language_context::LanguageContext;
use serial_test::serial;
use std::env;

//
// 1. Pure thread‑local cycle
//
#[test]
#[serial] // ensure isolation from other tests mutating env/thread-locals
fn test_thread_local_language_cycle() {
    // Make sure env fallback is not interfering
    env::remove_var("YEW_SSG_CURRENT_LANG");
    LanguageContext::clear_thread_local_lang();

    // Default fallback (no thread-local + no env)
    assert_eq!(LanguageContext::get_current_lang(), "en");

    // Set thread-local
    LanguageContext::set_thread_local_lang("fr");
    assert_eq!(LanguageContext::get_current_lang(), "fr");

    // Clear thread-local again
    LanguageContext::clear_thread_local_lang();
    assert_eq!(LanguageContext::get_current_lang(), "en");
}

//
// 2. Environment fallback when thread‑local is absent
//
#[test]
#[serial]
fn test_env_fallback_when_no_thread_local() {
    // Ensure clean thread-local
    LanguageContext::clear_thread_local_lang();

    env::set_var("YEW_SSG_CURRENT_LANG", "es");
    assert_eq!(LanguageContext::get_current_lang(), "es");

    // Thread-local still wins if set
    LanguageContext::set_thread_local_lang("de");
    assert_eq!(LanguageContext::get_current_lang(), "de");

    // After clearing TL, env fallback returns
    LanguageContext::clear_thread_local_lang();
    assert_eq!(LanguageContext::get_current_lang(), "es");

    // Cleanup
    env::remove_var("YEW_SSG_CURRENT_LANG");
    assert_eq!(LanguageContext::get_current_lang(), "en");
}
