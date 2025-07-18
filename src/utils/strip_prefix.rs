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
            if s.len() < prefix_len || !s[..prefix_len].eq_ignore_ascii_case(prefix) {
                return None;
            }
            if let Some(c) = &s[prefix_len..].chars().next() {
                if c.is_alphanumeric() {
                    return None;
                }
            }
            s = &s[prefix_len..];
        }
        Some(s)
    }
}
