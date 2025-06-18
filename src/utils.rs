pub trait StripPrefixIgnoreAsciiCase {
    fn strip_prefix_ignore_ascii_case(&self, prefix: &str) -> Option<&str>;
}
impl<T: AsRef<str> + ?Sized> StripPrefixIgnoreAsciiCase for T {
    fn strip_prefix_ignore_ascii_case(&self, prefix: &str) -> Option<&str> {
        let s = self.as_ref();
        let prefix_len = prefix.len();
        if s.len() >= prefix_len && s[..prefix_len].eq_ignore_ascii_case(prefix) {
            Some(&s[prefix_len..])
        } else {
            None
        }
    }
}
