use std::fmt;
use std::iter;
use crate::wtf8like::{IsWtf8Slice, IsWtf8Buf};

pub(crate) struct ArgsWtf8<S> {
    inner: std::vec::IntoIter<S>,
}

impl<S: IsWtf8Buf> ArgsWtf8<S> {
    pub(crate) fn parse_cmd<I: IsWtf8Slice + ?Sized>(input: &I) -> Self {
        let mut wide: Vec<_> = input.encode_wide();
        wide.push(0);

        ArgsWtf8 { inner: parse_lp_cmd_line(&wide).into_iter() }
    }
}

/// Implements the Windows command-line argument parsing algorithm.
///
/// Microsoft's documentation for the Windows CLI argument format can be found at
/// <https://docs.microsoft.com/en-us/previous-versions//17w5ykft(v=vs.85)>.
///
/// Windows includes a function to do this in shell32.dll,
/// but linking with that DLL causes the process to be registered as a GUI application.
/// GUI applications add a bunch of overhead, even if no windows are drawn. See
/// <https://randomascii.wordpress.com/2018/12/03/a-not-called-function-can-cause-a-5x-slowdown/>.
fn parse_lp_cmd_line<S: IsWtf8Buf>(
    lp_cmd_line: &[u16],
) -> Vec<S> {
    const BACKSLASH: u16 = '\\' as u16;
    const QUOTE: u16 = '"' as u16;
    const TAB: u16 = '\t' as u16;
    const SPACE: u16 = ' ' as u16;

    let mut ret_val = Vec::new();
    if lp_cmd_line[0] == 0 {
        ret_val.push(S::from_str("TEST.EXE"));
        return ret_val;
    }
    let mut cmd_line = {
        let mut end = 0;
        while lp_cmd_line[end] != 0 {
            end += 1;
        }
        &lp_cmd_line[..end]
    };
    // The executable name at the beginning is special.
    cmd_line = match cmd_line[0] {
        // The executable name ends at the next quote mark,
        // no matter what.
        QUOTE => {
            let args = {
                let mut cut = cmd_line[1..].splitn(2, |&c| c == QUOTE);
                if let Some(exe) = cut.next() {
                    ret_val.push(S::from_wide(exe));
                }
                cut.next()
            };
            if let Some(args) = args {
                args
            } else {
                return ret_val;
            }
        }
        // Implement quirk: when they say whitespace here,
        // they include the entire ASCII control plane:
        // "However, if lpCmdLine starts with any amount of whitespace, CommandLineToArgvW
        // will consider the first argument to be an empty string. Excess whitespace at the
        // end of lpCmdLine is ignored."
        0..=SPACE => {
            ret_val.push(S::from_str(""));
            &cmd_line[1..]
        },
        // The executable name ends at the next whitespace,
        // no matter what.
        _ => {
            let args = {
                let mut cut = cmd_line.splitn(2, |&c| c > 0 && c <= SPACE);
                if let Some(exe) = cut.next() {
                    ret_val.push(S::from_wide(exe));
                }
                cut.next()
            };
            if let Some(args) = args {
                args
            } else {
                return ret_val;
            }
        }
    };
    let mut cur = Vec::new();
    let mut in_quotes = false;
    let mut was_in_quotes = false;
    let mut backslash_count: usize = 0;
    for &c in cmd_line {
        match c {
            // backslash
            BACKSLASH => {
                backslash_count += 1;
                was_in_quotes = false;
            },
            QUOTE if backslash_count % 2 == 0 => {
                cur.extend(iter::repeat(b'\\' as u16).take(backslash_count / 2));
                backslash_count = 0;
                if was_in_quotes {
                    cur.push('"' as u16);
                    was_in_quotes = false;
                } else {
                    was_in_quotes = in_quotes;
                    in_quotes = !in_quotes;
                }
            }
            QUOTE if backslash_count % 2 != 0 => {
                cur.extend(iter::repeat(b'\\' as u16).take(backslash_count / 2));
                backslash_count = 0;
                was_in_quotes = false;
                cur.push(b'"' as u16);
            }
            SPACE | TAB if !in_quotes => {
                cur.extend(iter::repeat(b'\\' as u16).take(backslash_count));
                if !cur.is_empty() || was_in_quotes {
                    ret_val.push(S::from_wide(&cur[..]));
                    cur.truncate(0);
                }
                backslash_count = 0;
                was_in_quotes = false;
            }
            _ => {
                cur.extend(iter::repeat(b'\\' as u16).take(backslash_count));
                backslash_count = 0;
                was_in_quotes = false;
                cur.push(c);
            }
        }
    }
    cur.extend(iter::repeat(b'\\' as u16).take(backslash_count));
    // include empty quoted strings at the end of the arguments list
    if !cur.is_empty() || was_in_quotes || in_quotes {
        ret_val.push(S::from_wide(&cur[..]));
    }
    ret_val
}

pub(crate) struct ArgsInnerDebug<'a, S> {
    args: &'a ArgsWtf8<S>,
}

impl<'a, S: fmt::Debug> fmt::Debug for ArgsInnerDebug<'a, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.args.inner.as_slice().fmt(f)
    }
}

impl<S> ArgsWtf8<S> {
    pub(crate) fn inner_debug(&self) -> ArgsInnerDebug<'_, S> {
        ArgsInnerDebug {
            args: self
        }
    }
}

impl<S> Iterator for ArgsWtf8<S> {
    type Item = S;
    fn next(&mut self) -> Option<S> { self.inner.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

impl<S> DoubleEndedIterator for ArgsWtf8<S> {
    fn next_back(&mut self) -> Option<S> { self.inner.next_back() }
}

impl<S> ExactSizeIterator for ArgsWtf8<S> {
    fn len(&self) -> usize { self.inner.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wtf8::Wtf8Buf;

    fn chk(string: &str, parts: &[&str]) {
        let mut wide: Vec<u16> = Wtf8Buf::from_str(string).to_ill_formed_utf16().collect();
        wide.push(0);
        let parsed = parse_lp_cmd_line::<Wtf8Buf>(&wide);
        let expected: Vec<Wtf8Buf> = parts.iter().map(|k| Wtf8Buf::from_str(k)).collect();
        assert_eq!(parsed.as_slice(), expected.as_slice());
    }

    #[test]
    fn empty() {
        chk("", &["TEST.EXE"]);
        chk("\0", &["TEST.EXE"]);
    }

    #[test]
    fn single_words() {
        chk("EXE one_word", &["EXE", "one_word"]);
        chk("EXE a", &["EXE", "a"]);
        chk("EXE 😅", &["EXE", "😅"]);
        chk("EXE 😅🤦", &["EXE", "😅🤦"]);
    }

    #[test]
    fn official_examples() {
        chk(r#"EXE "abc" d e"#, &["EXE", "abc", "d", "e"]);
        chk(r#"EXE a\\\b d"e f"g h"#, &["EXE", r#"a\\\b"#, "de fg", "h"]);
        chk(r#"EXE a\\\"b c d"#, &["EXE", r#"a\"b"#, "c", "d"]);
        chk(r#"EXE a\\\\"b c" d e"#, &["EXE", r#"a\\b c"#, "d", "e"]);
    }

    #[test]
    fn whitespace_behavior() {
        chk(r#" test"#, &["", "test"]);
        chk(r#"  test"#, &["", "test"]);
        chk(r#" test test2"#, &["", "test", "test2"]);
        chk(r#" test  test2"#, &["", "test", "test2"]);
        chk(r#"test test2 "#, &["test", "test2"]);
        chk(r#"test  test2 "#, &["test", "test2"]);
        chk(r#"test "#, &["test"]);
    }

    #[test]
    fn genius_quotes() {
        chk(r#"EXE "" """#, &["EXE", "", ""]);
        chk(r#"EXE "" """"#, &["EXE", "", "\""]);
        chk(
            r#"EXE "this is """all""" in the same argument""#,
            &["EXE", "this is \"all\" in the same argument"]
        );
        chk(r#"EXE "a"""#, &["EXE", "a\""]);
        chk(r#"EXE "a"" a"#, &["EXE", "a\"", "a"]);
        // quotes cannot be escaped in command names
        chk(r#""EXE" check"#, &["EXE", "check"]);
        chk(r#""EXE check""#, &["EXE check"]);
        chk(r#""EXE """for""" check"#, &["EXE ", r#"for""#, "check"]);
        chk(r#""EXE \"for\" check"#, &[r#"EXE \"#, r#"for""#,  "check"]);
    }
}
