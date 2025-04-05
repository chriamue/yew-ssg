pub fn get_base_url() -> String {
    std::env::var("BASE_URL").unwrap_or_default()
}

pub fn combine_with_base_url(path: &str) -> String {
    if let Ok(base_url) = std::env::var("BASE_URL") {
        let base_url = base_url.trim_end_matches('/');
        let path_cleaned = path.trim_start_matches('/');
        format!("{}/{}", base_url, path_cleaned)
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_get_base_url() {
        // First test when BASE_URL is not set
        std::env::remove_var("BASE_URL");
        assert_eq!(get_base_url(), "");

        // Then test when BASE_URL is set
        std::env::set_var("BASE_URL", "/base");
        assert_eq!(get_base_url(), "/base");
    }

    #[test]
    #[serial]
    fn test_combine_with_base_url() {
        // Test without BASE_URL
        std::env::remove_var("BASE_URL");
        assert_eq!(combine_with_base_url("/page"), "/page");

        // Test with BASE_URL
        std::env::set_var("BASE_URL", "/base");
        assert_eq!(combine_with_base_url("/page"), "/base/page");
        assert_eq!(combine_with_base_url("page"), "/base/page");

        // Test with trailing slash in BASE_URL
        std::env::set_var("BASE_URL", "/base/");
        assert_eq!(combine_with_base_url("/page"), "/base/page");

        // Cleanup
        std::env::remove_var("BASE_URL");
    }
}
