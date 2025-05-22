use crate::*;

pub fn is_atext(input: &[u8]) -> bool {
    if input.is_empty() {
        return false;
    }

    input.iter().all(|&c| {
        c.is_ascii_alphanumeric()
            || matches!(
                c,
                b'!'
                | b'#'..=b'\''
                | b'*'..=b'+'
                | b'-' | b'/' | b'=' | b'?' | b'^' | b'_' | b'`'
                | b'{'..=b'}'
            )
    })
}

pub fn is_dot_string(input: &[u8]) -> bool {
    let (a, b) = input.split_once_str(".").unwrap_or((input, b""));

    if !is_atext(a) {
        return false;
    }

    if b.is_empty() {
        return true;
    }

    b.split(|&x| x == b'.').all(is_atext)
}

pub fn is_qtext(input: u8) -> bool {
    matches!(input, b' '..=b'!' |  b'#'..=b'[' | b']'..=b'~')
}

pub fn is_quoted_pair(input: u8) -> bool {
    matches!(input, b' '..=b'~')
}

pub fn is_quoted_string(input: &[u8]) -> bool {
    let Some(stripped) = strip_quotes(input) else {
        return false;
    };

    let mut i = 0;
    while i < stripped.len() {
        if stripped[i] == b'\\' {
            if i + 1 < stripped.len() && is_quoted_pair(stripped[i + 1]) {
                i += 2;
                continue;
            }
            return false;
        } else if !is_qtext(stripped[i]) {
            return false;
        }
        i += 1;
    }
    true
}

pub fn is_subdomain(input: &[u8]) -> bool {
    if input.is_empty() {
        return false;
    }

    if input[0] == b'-' || input[input.len() - 1] == b'-' {
        return false;
    }

    input
        .iter()
        .all(|&c| c.is_ascii_alphanumeric() || c == b'-')
}

pub fn is_domain(input: &[u8]) -> bool {
    let (a, b) = input.split_once_str(".").unwrap_or((input, &[]));

    if !is_subdomain(a) {
        return false;
    }

    if b.is_empty() {
        return true;
    }

    b.split(|&x| x == b'.').all(is_subdomain)
}

#[cfg_attr(test, mutants::skip)]
pub fn is_local_part(input: &[u8]) -> bool {
    is_dot_string(input) || is_quoted_string(input)
}

pub fn strip_quotes(input: &[u8]) -> Option<&[u8]> {
    input.strip_prefix(b"\"")?.strip_suffix(b"\"")
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::alphanum(b"abcABC123", true)]
    #[case::special(b"!#$%&'*+-/=?^_`{|}", true)]
    #[case::mixed(b"mixed123!#$", true)]
    #[case::empty(b"", false)]
    #[case::space(b"hello world", false)]
    #[case::control(b"hello\x00world", false)]
    #[case::atom(b"hello@world", false)]
    #[case::comma(b"hello,world", false)]
    #[case::quote(b"hello\"world", false)]
    #[case::tab(b"hello\tworld", false)]
    #[case::non_ascii(b"hello\x80world", false)]
    fn test_is_atext(#[case] input: &[u8], #[case] expected: bool) {
        assert_eq!(is_atext(input), expected);
    }

    #[rstest]
    #[case::simple(b"simple", true)]
    #[case::one_dot(b"with.dot", true)]
    #[case::multiple_dots(b"with.multiple.dots", true)]
    #[case::special(b"special!#$%.chars", true)]
    #[case::empty(b"", false)]
    #[case::dot_only(b".", false)]
    #[case::leading_dot(b".leading", false)]
    #[case::double_dot(b"double..dot", false)]
    #[case::space(b"with space.com", false)]
    #[case::illegal_chars(b"illegal@.char", false)]
    #[case::non_ascii(b"hello.\x80world", false)]
    fn test_is_dot_string(#[case] input: &[u8], #[case] expected: bool) {
        assert_eq!(is_dot_string(input), expected);
    }

    #[rstest]
    #[case::space(b' ', true)]
    #[case::uppercase(b'A', true)]
    #[case::lowercase(b'z', true)]
    #[case::exclamation(b'!', true)]
    #[case::non_ascii(b'\x80', false)]
    #[case::control(b'\x00', false)]
    fn test_is_qtext(#[case] input: u8, #[case] expected: bool) {
        assert_eq!(is_qtext(input), expected);
    }

    #[rstest]
    #[case::backslash(b'\\', true)]
    #[case::space(b' ', true)]
    #[case::tilde(b'~', true)]
    #[case::non_ascii(b'\x80', false)]
    #[case::control(b'\0', false)]
    #[case::special(b'!', true)]
    #[case::uppercase(b'A', true)]
    #[case::lowercase(b'z', true)]
    #[case::exclamation(b'!', true)]
    #[case::quote(b'"', true)]
    fn test_is_quoted_pair(#[case] input: u8, #[case] expected: bool) {
        assert_eq!(is_quoted_pair(input), expected);
    }

    #[rstest]
    #[case::simple(b"\"quoted\"", true)]
    #[case::space(b"\"quoted with space\"", true)]
    #[case::special(b"\"quoted!#$%&'*+-/=?^_`{|}\"", true)]
    #[case::escaped_quote(b"\"quoted\\\"\"", true)]
    #[case::escaped_backslash(b"\"quoted\\\\\"", true)]
    #[case::not_quoted(b"not quoted", false)]
    #[case::open_quote(b"\"open quoted", false)]
    #[case::close_quote(b"close quoted\"", false)]
    #[case::escape_at_end(b"\"quoted\\\"", false)]
    #[case::unescaped_quote(b"quoted\"text", false)]
    #[case::invalid_escape(b"\"test\\\x7f\"", false)]
    #[case::non_ascii(b"\"quoted\x80\"", false)]
    #[case::empty(b"\"\"", true)]
    #[case::multiple_quotes(b"\"test\\\"test\\\"again\"", true)]
    #[case::single_backslash(b"\"\\\\\"", true)]
    #[case::newline(b"\"\\n\"", true)]
    #[case::alpha(b"\"abcdefghijklmnopqrstuvwxyz\"", true)]
    #[case::complex(b"\"a\\\"b\\\"c\\\"d\\\"e\"", true)]
    fn test_is_quoted_string(#[case] input: &[u8], #[case] expected: bool) {
        assert_eq!(is_quoted_string(input), expected);
    }

    #[rstest]
    #[case::simple(b"simple", true)]
    #[case::hyphenated(b"hyphen-ated", true)]
    #[case::mixed(b"mixed-123", true)]
    #[case::empty(b"", false)]
    #[case::leading_hyphen(b"-leading", false)]
    #[case::trailing_hyphen(b"trailing-", false)]
    #[case::multiple_hyphens(b"multiple--hyphens", true)]
    #[case::space(b"with space", false)]
    #[case::dot(b"with.dot", false)]
    #[case::underscore(b"with_underscore", false)]
    fn test_is_subdomain(#[case] input: &[u8], #[case] expected: bool) {
        assert_eq!(is_subdomain(input), expected);
    }

    #[rstest]
    #[case::simple(b"simple", true)]
    #[case::hyphenated(b"hyphen-ated", true)]
    #[case::mixed(b"mixed-123", true)]
    #[case::empty(b"", false)]
    #[case::leading_hyphen(b"-leading", false)]
    #[case::trailing_hyphen(b"trailing-", false)]
    #[case::multiple_hyphens(b"multiple--hyphens", true)]
    #[case::space(b"with space", false)]
    #[case::dot(b"with.dot", true)]
    #[case::underscore(b"with_underscore", false)]
    #[case::leading_dot(b".leading", false)]
    #[case::trailing_dot(b"trailing.", true)]
    #[case::multiple_dots(b"multiple..dots", false)]
    #[case::subdomain_leading_hyphen(b"subdomain.-leading.com", false)]
    #[case::subdomain_trailing_hyphen(b"subdomain.trailing-.com", false)]
    fn test_is_domain(#[case] input: &[u8], #[case] expected: bool) {
        assert_eq!(is_domain(input), expected);
    }

    #[rstest]
    #[case::prefix(b"prefix", b"pre", Some(b"fix".as_slice()))]
    #[case::case_insensitive(b"PrEfIx", b"pre", Some(b"fIx".as_slice()))]
    #[case::not_found(b"prefix", b"foo", None)]
    #[case::empty_prefix(b"prefix", b"", Some(b"prefix".as_slice()))]
    #[case::empty_input(b"", b"prefix", None)]
    #[case::empty_both(b"", b"", Some(b"".as_slice()))]
    #[case::longer_prefix(b"prefix", b"prefixes", None)]
    fn test_strip_prefix_ci(
        #[case] input: &'static [u8],
        #[case] prefix: &'static [u8],
        #[case] expected: Option<&'static [u8]>,
    ) {
        assert_eq!(
            Bytes::from(input).strip_prefix_ci(prefix),
            expected.map(Bytes::from)
        );
    }

    #[rstest]
    #[case::bang(b'!', true)]
    #[case::asterisk(b'*', true)]
    #[case::comma(b',', true)]
    #[case::lt(b'<', true)]
    #[case::gt(b'>', true)]
    #[case::tilde(b'~', true)]
    #[case::space(b' ', false)]
    #[case::plus(b'+', false)]
    #[case::equals(b'=', false)]
    #[case::lf(b'\n', false)]
    #[case::del(b'\x7f', false)]
    #[case::non_ascii(b'\x80', false)]
    fn test_is_xchar(#[case] input: u8, #[case] expected: bool) {
        assert_eq!(is_xchar(input), expected);
    }
}
