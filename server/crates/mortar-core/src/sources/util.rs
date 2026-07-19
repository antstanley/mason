//! Small shared helpers for the source layer.

/// Whether a URL is safe to hand a brick, which is to say safe to reach an
/// `<a href>` in the browser: only http and https. Third-party records carry
/// arbitrary strings in their url fields, and `javascript:`, `data:`, and
/// `vbscript:` URLs must never survive the trip to the anchor.
pub fn is_http_url(url: &str) -> bool {
    let lower = url.trim_start().to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_http_and_https_pass() {
        assert!(is_http_url("https://example.com/post"));
        assert!(is_http_url("http://example.com"));
        assert!(is_http_url("  HTTPS://example.com")); // trimmed, case-insensitive
        assert!(!is_http_url("javascript:alert(1)"));
        assert!(!is_http_url("data:text/html,<script>"));
        assert!(!is_http_url("vbscript:msgbox"));
        assert!(!is_http_url("//example.com"));
        assert!(!is_http_url(""));
    }
}
