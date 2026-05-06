pub fn format_for_claude(text: &str, repo: &str, query_type: &str) -> String {
    format!("## CodeWiki: {} ({})\n\n{}", repo, query_type, text.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_adds_header() {
        let result = format_for_claude("some content", "facebook/react", "ask");
        assert!(result.starts_with("## CodeWiki: facebook/react (ask)"));
        assert!(result.contains("some content"));
    }

    #[test]
    fn format_trims_whitespace() {
        let result = format_for_claude("  content  \n\n", "owner/repo", "structure");
        assert!(result.ends_with("content"));
    }

    #[test]
    fn format_handles_empty_text() {
        let result = format_for_claude("   \n\t  ", "owner/repo", "read");
        assert_eq!(result, "## CodeWiki: owner/repo (read)\n\n");
    }
}
