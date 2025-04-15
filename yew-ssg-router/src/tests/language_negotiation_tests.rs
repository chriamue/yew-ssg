#![cfg(test)]

use crate::LanguageNegotiator;

#[test]
fn test_language_negotiator() {
    let negotiator = LanguageNegotiator::from_static(&["en", "fr", "de"], "en");

    // Test with exact match
    let result = negotiator.negotiate_from_header(Some("fr,en;q=0.8,de;q=0.6"));
    assert_eq!(result, "fr");

    // Test with quality preference
    let result = negotiator.negotiate_from_header(Some("en;q=0.8,fr;q=0.9,de;q=0.7"));
    assert_eq!(result, "fr");

    // Test with language tag (e.g., en-US)
    let result = negotiator.negotiate_from_header(Some("en-US,en;q=0.9"));
    assert_eq!(result, "en");

    // Test with no match
    let result = negotiator.negotiate_from_header(Some("es,it"));
    assert_eq!(result, "en"); // Default language

    // Test with empty header
    let result = negotiator.negotiate_from_header(None);
    assert_eq!(result, "en"); // Default language
}

#[test]
fn test_parse_accept_language() {
    let negotiator = LanguageNegotiator::from_static(&["en", "fr"], "en");

    // Access the private method through negotiate_from_header
    // We just verify it works properly through the results

    // Test basic parsing through the results
    let result = negotiator.negotiate_from_header(Some("fr;q=0.7,en;q=0.8"));
    assert_eq!(result, "en"); // en has higher q value

    // Test with malformed q values
    let result = negotiator.negotiate_from_header(Some("fr;q=invalid,en"));
    assert_eq!(result, "en"); // fr's q value is invalid, so en wins
}
