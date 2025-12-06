/// Trait providing case-insensitive stripping of word prefixes from a string.
///
/// This is useful for checking SQL keywords like `NOT IN (...)`
/// while ignoring case and ensuring prefix boundaries are respected.
pub trait StripPrefixWordIgnoreAsciiCase {
    /// Strips a sequence of words from the beginning of a string,
    /// ignoring ASCII case and ensuring each word is not a prefix of a larger identifier.
    ///
    /// Returns the remainder of the string if matched.
    fn strip_prefix_word_ignore_ascii_case(&self, prefix_words: &[&str]) -> Option<&str>;
}
impl<T: AsRef<str> + ?Sized> StripPrefixWordIgnoreAsciiCase for T {
    fn strip_prefix_word_ignore_ascii_case(&self, prefix_words: &[&str]) -> Option<&str> {
        let mut s = self.as_ref();
        for (i, prefix) in prefix_words.iter().enumerate() {
            if i > 0 {
                s = s.trim_start();
            }

            let prefix_len = prefix.len();
            if s.len() < prefix_len
                || !s.is_char_boundary(prefix_len)
                || !s[..prefix_len].eq_ignore_ascii_case(prefix)
            {
                return None;
            }
            if let Some(c) = &s[prefix_len..].chars().next()
                && !c.is_whitespace() && !c.is_ascii_punctuation()
            {
                return None;
            }
            s = &s[prefix_len..];
        }
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use super::StripPrefixWordIgnoreAsciiCase;

    #[test]
    pub fn test_strip_prefix_word_ignore_ascii_case() {
        use crate::utils::strip_prefix::StripPrefixWordIgnoreAsciiCase;
        "NOT IN (:ph)"
            .strip_prefix_word_ignore_ascii_case(&["NOT", "IN"])
            .unwrap();
    }

    #[test]
    fn test_single_word_exact() {
        assert_eq!(
            "SELECT * FROM users".strip_prefix_word_ignore_ascii_case(&["SELECT"]),
            Some(" * FROM users")
        );
    }

    #[test]
    fn test_single_word_with_case_variants() {
        assert_eq!(
            "select *".strip_prefix_word_ignore_ascii_case(&["SELECT"]),
            Some(" *")
        );
        assert_eq!(
            "SeLeCt *".strip_prefix_word_ignore_ascii_case(&["SELECT"]),
            Some(" *")
        );
    }

    #[test]
    fn test_multi_word_exact() {
        assert_eq!(
            "NOT IN (1, 2, 3)".strip_prefix_word_ignore_ascii_case(&["NOT", "IN"]),
            Some(" (1, 2, 3)")
        );
    }

    #[test]
    fn test_multi_word_case_variants() {
        assert_eq!(
            "not in (1, 2, 3)".strip_prefix_word_ignore_ascii_case(&["NOT", "IN"]),
            Some(" (1, 2, 3)")
        );
        assert_eq!(
            "Not In (1)".strip_prefix_word_ignore_ascii_case(&["NOT", "IN"]),
            Some(" (1)")
        );
    }

    #[test]
    fn test_no_match_due_to_partial_word() {
        assert_eq!(
            "SELECTED *".strip_prefix_word_ignore_ascii_case(&["SELECT"]),
            None
        );
        assert_eq!(
            "NOTIFY something".strip_prefix_word_ignore_ascii_case(&["NOT"]),
            None
        );
    }

    #[test]
    fn test_no_match_due_to_wrong_order() {
        assert_eq!(
            "IN NOT (...)".strip_prefix_word_ignore_ascii_case(&["NOT", "IN"]),
            None
        );
    }

    #[test]
    fn test_leading_whitespace_skipped_only_after_first_word() {
        assert_eq!(
            "NOT    IN (1)".strip_prefix_word_ignore_ascii_case(&["NOT", "IN"]),
            Some(" (1)")
        );
        assert_eq!(
            "NOTIN (1)".strip_prefix_word_ignore_ascii_case(&["NOT", "IN"]),
            None
        );
    }

    #[test]
    fn test_empty_prefix_list() {
        assert_eq!(
            "SELECT *".strip_prefix_word_ignore_ascii_case(&[]),
            Some("SELECT *")
        );
    }

    #[test]
    fn test_exact_prefix_only() {
        assert_eq!("IN".strip_prefix_word_ignore_ascii_case(&["IN"]), Some(""));
        assert_eq!("in".strip_prefix_word_ignore_ascii_case(&["IN"]), Some(""));
        assert_eq!("index".strip_prefix_word_ignore_ascii_case(&["IN"]), None); // should fail
    }
}
