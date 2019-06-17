#![doc(html_root_url = "https://docs.rs/windows-args/0.1.0")]

//! # `windows-args`
//!
//! A command-line argument parser for Windows, copied almost wholesale from the rust standard library.
//!
//! ## Parsing the command line
//!
//! There are two types, which slightly differ in how they parse input:
//!
//! * [`Command`] parses a complete command line string, including the executable.
//! * [`Args`] parses whitespace-separated arguments, without the executable.
//!
//! ```rust
//! use windows_args::Command;
//!
//! // parse a complete command string (beginning with an executable name)
//! let cmd = Command::parse(r#"foobar.exe to "C:\Program Files\Hi.txt" now"#);
//!
//! assert_eq!(cmd.len(), 4);
//! assert_eq!(cmd.exe, "foobar.exe");
//! assert_eq!(cmd.iter().nth(0), Some("foobar.exe"));
//! assert_eq!(cmd.into_iter().nth(2), Some("C:\\Program Files\\Hi.txt".to_string()));
//! ```
#![cfg_attr(windows, doc = "\
## `OsString` support

Exclusive to Windows platforms are the types [`CommandOs`] and [`ArgsOs`], which provide
[`std::ffi::OsString`] support.
")]
#![cfg_attr(not(windows), doc = "\
## `OsString` support

Exclusive to Windows platforms are the types `CommandOs` and `ArgsOs`, which provide
[`std::ffi::OsString`] support.
")]

#[cfg(windows)]
use std::ffi::{OsStr, OsString};
use std::fmt;
use crate::args::ArgsWtf8;
use wtf8::{Wtf8, Wtf8Buf};

mod wtf8like;
mod args;

pub use crate::iter::{Iter, IntoIter};
#[cfg(windows)]
pub use crate::iter::{IterOs, IntoIterOs};
mod iter;

/// Arguments to a process (not including the executable), stored as [`String`]s.
#[derive(Clone, PartialEq, Eq)]
pub struct Args { inner: ArgsWtf8<Wtf8Buf> }

/// Arguments to a process (not including the executable), stored as [`OsString`]s.
#[cfg(windows)]
#[derive(Clone, PartialEq, Eq)]
pub struct ArgsOs { inner: ArgsWtf8<OsString> }

/// A parsed command-line string, including the executable, stored as [`String`]s.
///
/// This type is iterable; iterating over it produces the executable path,
/// followed by the arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub exe: String,
    pub args: Args,
}

/// A parsed command-line string, including the executable, stored as [`OsString`]s.
///
/// This type is iterable; iterating over it produces the executable path,
/// followed by the arguments.
#[cfg(windows)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOs {
    pub exe: OsString,
    pub args: ArgsOs,
}

impl Command {
    /// Parse a string containing a complete command line.
    ///
    /// The behavior is identical to [`CommandLineToArgvW`], with one minor exception for the
    /// empty input (which now produces `[""]` rather than the current executable).
    ///
    /// [`CommandLineToArgvW`]: https://docs.microsoft.com/en-us/windows/desktop/api/shellapi/nf-shellapi-commandlinetoargvw
    ///
    /// ```
    /// let args = windows_args::Command::parse(r#"me.exe  \\\"#);
    /// assert_eq!(
    ///     args.into_iter().collect::<Vec<_>>(),
    ///     vec!["me.exe".to_string(), r#"\\\"#.to_string()],
    /// );
    /// ```
    pub fn parse(input: &str) -> Self {
        let mut args = ArgsWtf8::parse_cmd(Wtf8::from_str(input));
        let exe = expect_still_utf8_own(args.vec.remove(0));
        let args = Args { inner: args };
        Command { exe, args }
    }

    /// Get the length of ARGV, including the executable.
    pub fn len(&self) -> usize {
        self.args.len() + 1
    }

    /// Iterate over the arguments, including the executable.
    ///
    /// Item type is `&str`.
    pub fn iter(&self) -> Iter<'_> {
        Iter::from_cmd(self)
    }
}

#[cfg(windows)]
impl CommandOs {
    /// Parse an [`OsStr`] containing the complete command line.
    ///
    /// The behavior is identical to [`CommandLineToArgvW`], with one minor exception for the
    /// empty input (which now produces `[""]` rather than the current executable).
    ///
    /// [`CommandLineToArgvW`]: https://docs.microsoft.com/en-us/windows/desktop/api/shellapi/nf-shellapi-commandlinetoargvw
    ///
    /// ```rust
    /// use std::ffi::OsString;
    ///
    /// let args = windows_args::CommandOs::parse("test  \" \"".as_ref());
    /// assert_eq!(
    ///     args.into_iter().collect::<Vec<_>>(),
    ///     vec!["test".into(), " ".into()] as Vec<OsString>,
    /// );
    /// ```
    pub fn parse(input: &OsStr) -> Self {
        let mut args = ArgsWtf8::parse_cmd(input);
        let exe = args.vec.remove(0);
        let args = ArgsOs { inner: args };
        CommandOs { exe, args }
    }

    /// Get the length of ARGV, including the executable.
    pub fn len(&self) -> usize {
        self.args.len() + 1
    }

    /// Iterate over the arguments, including the executable.
    ///
    /// Item type is `&OsStr`.
    pub fn iter(&self) -> IterOs<'_> {
        IterOs::from_cmd(self)
    }
}

impl Args {
    /// Parse a string containing whitespace-separated arguments to an executable.
    ///
    /// This function is intended to be used for strings which **do not** begin with
    /// the executable name.
    ///
    /// ```
    /// let args = windows_args::Args::parse(r#"file.txt  \\\"#);
    /// assert_eq!(
    ///     args.into_iter().collect::<Vec<_>>(),
    ///     vec!["file.txt".to_string(), r#"\\\"#.to_string()],
    /// );
    /// ```
    pub fn parse(input: &str) -> Self {
        parse_args_via_parse_cmd(
            input,
            Command::parse,
            String::with_capacity,
            String::push_str,
            str::len,
            |cmd| cmd.args,
        )
    }

    /// Get the number of arguments.
    pub fn len(&self) -> usize {
        self.inner.vec.len()
    }

    /// Iterate over the arguments.
    ///
    /// Item type is `&str`.
    pub fn iter(&self) -> Iter<'_> {
        Iter::from_args(self)
    }
}

#[cfg(windows)]
impl ArgsOs {
    /// Parse an [`OsStr`] containing whitespace-separated arguments to an executable.
    ///
    /// This function is intended to be used for strings which **do not** begin with
    /// the executable name.
    ///
    /// ```rust
    /// use std::ffi::OsString;
    ///
    /// let args = windows_args::ArgsOs::parse("test  \" \"".as_ref());
    /// assert_eq!(
    ///     args.into_itercollect::<Vec<_>>(),
    ///     vec!["test".into(), " ".into()] as Vec<OsString>,
    /// );
    /// ```
    pub fn parse(input: &OsStr) -> Self {
        parse_args_via_parse_cmd(
            input,
            CommandOs::parse,
            OsString::with_capacity,
            |buf, s| buf.push(s),
            OsStr::len,
            |cmd| cmd.args,
        )
    }

    /// Get the number of arguments.
    pub fn len(&self) -> usize {
        self.inner.vec.len()
    }

    /// Iterate over the arguments.
    ///
    /// Item type is `&OsStr`.
    pub fn iter(&self) -> IterOs<'_> {
        IterOs::from_args(self)
    }
}

impl IntoIterator for Command {
    type Item = String;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::from_cmd(self)
    }
}

#[cfg(windows)]
impl IntoIterator for CommandOs {
    type Item = OsString;
    type IntoIter = IntoIterOs;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterOs::from_cmd(self)
    }
}

impl IntoIterator for Args {
    type Item = String;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::from_args(self)
    }
}

#[cfg(windows)]
impl IntoIterator for ArgsOs {
    type Item = OsString;
    type IntoIter = IntoIterOs;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterOs::from_args(self)
    }
}

fn expect_still_utf8_own(arg: Wtf8Buf) -> String {
    arg.into_string().unwrap_or_else(|arg| {
        panic!("\
valid UTF-8 became invalid after arg splitting?!
BadArg: {:?}\
", arg);
    })
}

fn expect_still_utf8_ref(arg: &Wtf8Buf) -> &str {
    arg.as_str().unwrap_or_else(|| {
        panic!("\
valid UTF-8 became invalid after arg splitting?!
BadArg: {:?}\
", arg);
    })
}

impl fmt::Debug for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Args")
            .field("vec", &&self.inner.vec[..])
            .finish()
    }
}

#[cfg(windows)]
impl fmt::Debug for ArgsOs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArgsOs")
            .field("vec", &&self.inner.vec[..])
            .finish()
    }
}

fn parse_args_via_parse_cmd<Args, Command, OwnS, RefS: ?Sized>(
    input: &RefS,
    parse_cmd: impl FnOnce(&RefS) -> Command,
    with_capacity: impl FnOnce(usize) -> OwnS,
    push_str: impl Fn(&mut OwnS, &RefS),
    len: impl Fn(&RefS) -> usize,
    project_args: impl FnOnce(Command) -> Args
) -> Args
where
    OwnS: std::ops::Deref<Target=RefS>,
    str: AsRef<RefS>,
{
    // Prepend a command name
    let mut modified_input = with_capacity(len(input) + 2);
    push_str(&mut modified_input, "a ".as_ref());
    push_str(&mut modified_input, input);
    project_args(parse_cmd(&modified_input))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special_traits() {
        assert_eq!(Command::parse("a b").into_iter().next_back(), Some("b".into()));
        assert_eq!(Command::parse("a b").iter().next_back(), Some("b"));
        assert_eq!(Args::parse("a b").into_iter().next_back(), Some("b".into()));
        assert_eq!(Args::parse("a b").iter().next_back(), Some("b"));
    }

    #[cfg(windows)]
    #[test]
    fn special_traits_windows() {
        assert_eq!(CommandOs::parse("a b".as_ref()).into_iter().next_back(), Some("b".into()));
        assert_eq!(CommandOs::parse("a b".as_ref()).iter().next_back(), Some("b".as_ref()));
        assert_eq!(ArgsOs::parse("a b".as_ref()).into_iter().next_back(), Some("b".into()));
        assert_eq!(ArgsOs::parse("a b".as_ref()).iter().next_back(), Some("b".as_ref()));
    }

    #[test]
    fn args_cmd_differences() {
        assert_eq!(Command::parse("").into_iter().collect::<Vec<_>>(), vec![String::new()]);
        assert_eq!(Args::parse("").into_iter().collect::<Vec<_>>(), Vec::<String>::new());
        assert_eq!(Command::parse("  ").into_iter().collect::<Vec<_>>(), vec![String::new()]);
        assert_eq!(Args::parse("  ").into_iter().collect::<Vec<_>>(), Vec::<String>::new());

        assert_eq!(
            Command::parse(r#""abc\"def""#).into_iter().collect::<Vec<_>>(),
            vec!["abc\\".to_string(), "def".to_string(),
        ]);
        assert_eq!(
            Args::parse(r#""abc\"def""#).into_iter().collect::<Vec<_>>(),
            vec!["abc\"def".to_string()],
        );

        assert_eq!(
            Command::parse(r#"a "abc\"def""#).into_iter().collect::<Vec<_>>(),
            vec!["a".to_string(), "abc\"def".to_string()],
        );
        assert_eq!(
            Args::parse(r#"a "abc\"def""#).into_iter().collect::<Vec<_>>(),
            vec!["a".to_string(), "abc\"def".to_string()],
        );
    }
}
