#[derive(Clone)]
pub struct StatAchievementWorkaround {
    pub statistic_key: String,
    pub achievement_key: String,
    pub threshold: String, // Since we dont know the type of the stat we dont parse it
}

pub struct QuotedSplit<'a> {
    buffer: &'a str,
}

impl<'a> QuotedSplit<'a> {
    pub fn new(buffer: &'a str) -> Self {
        Self { buffer }
    }
}

impl<'a> Iterator for QuotedSplit<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let s = self.buffer.trim_start();
        if s.is_empty() {
            self.buffer = s;
            return None;
        }

        let mut out = String::new();
        let mut quote: Option<char> = None;
        let mut end = s.len();
        let mut chars = s.char_indices();

        while let Some((i, c)) = chars.next() {
            match (quote, c) {
                (None, c) if c.is_whitespace() => {
                    end = i;
                    break;
                }
                (None, '"') => quote = Some(c),
                // Closing quote.
                (Some(q), c) if c == q => quote = None,
                // Escape
                (None | Some('"'), '\\') => match chars.next() {
                    Some((_, next)) => out.push(next),
                    None => out.push('\\'), // trailing backslash: keep literally
                },
                _ => out.push(c),
            }
        }

        self.buffer = &s[end..];
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Vec<String> {
        QuotedSplit::new(s).collect()
    }

    #[test]
    fn plain() {
        assert_eq!(v("a b  c"), ["a", "b", "c"]);
        assert!(v("   ").is_empty());
        assert!(v("").is_empty());
    }

    #[test]
    fn mid_token_and_empty() {
        assert_eq!(v(r#"foo"bar baz"qux"#), ["foobar bazqux"]);
        assert_eq!(v(r#"a "" b"#), ["a", "", "b"]);
    }

    #[test]
    fn escapes() {
        assert_eq!(v(r#"a\ b c"#), ["a b", "c"]);
        assert_eq!(v(r#""a \"b\" c""#), [r#"a "b" c"#]);
        assert_eq!(v(r"trailing\"), [r"trailing\"]);
    }

    #[test]
    fn unterminated() {
        assert_eq!(v(r#"a "b c"#), ["a", "b c"]);
    }

    #[test]
    fn unicode() {
        assert_eq!(v("żółć \"gęślą jaźń\""), ["żółć", "gęślą jaźń"]);
    }
}
