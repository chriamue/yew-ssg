pub fn strip_slash_suffix(path: &str) -> &str {
    path.strip_suffix('/').unwrap_or(path)
}
