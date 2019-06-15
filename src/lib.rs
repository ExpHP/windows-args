//! # `windows-args`
//!
//! A command-line argument parser for Windows, copied almost wholesale from the rust standard library.
//!
//! ```rust
//! use windows_args::Args;
//!
//! let mut args = Args::parse_cmd(r#"foobar to "C:\Program Files\Hi.txt" now"#);
//! assert_eq!(args.next(), Some("foobar".to_string()));
//! assert_eq!(args.next(), Some("to".to_string()));
//! assert_eq!(args.next(), Some("C:\\Program Files\\Hi.txt".to_string()));
//! assert_eq!(args.next(), Some("now".to_string()));
//! assert_eq!(args.next(), None);
//! ```

use std::ffi::{OsStr, OsString};
use std::fmt;

mod wtf8;
mod args;

/// An iterator over the arguments of a process, yielding a [`String`] value for
/// each argument.
///
/// The first element is traditionally the path of the executable, but it can be
/// set to arbitrary text, and may not even exist. This means this property
/// should not be relied upon for security purposes.
///
/// [`String`]: ../string/struct.String.html
pub struct Args { inner: std::vec::IntoIter<String> }

/// An iterator over the arguments of a process, yielding an [`OsString`] value
/// for each argument.
///
/// The first element is traditionally the path of the executable, but it can be
/// set to arbitrary text, and may not even exist. This means this property
/// should not be relied upon for security purposes.
///
/// [`OsString`]: ../ffi/struct.OsString.html
pub struct ArgsOs { inner: crate::args::Args }

impl ArgsOs {
    /// Parse an OsStr containing the complete command line.
    ///
    /// The output will always contain at least one argument (representing the executable name).
    /// If the input was empty, a placeholder name is given.
    ///
    /// **This function is not suitable for strings that do not contain an executable name.**
    ///
    /// ```rust
    /// use std::ffi::OsString;
    ///
    /// let args = windows_args::ArgsOs::parse_cmd("test  \" \"".as_ref());
    /// assert_eq!(
    ///     args.collect::<Vec<_>>(),
    ///     vec!["test".into(), " ".into()] as Vec<OsString>,
    /// );
    /// ```
    pub fn parse_cmd(arg_str: &OsStr) -> Self {
        ArgsOs { inner: crate::args::Args::parse(arg_str) }
    }
}

impl Args {
    /// Parse a string containing the complete command line.
    ///
    /// The output will always contain at least one argument (representing the executable name).
    /// If the input was empty, a placeholder name is given.
    ///
    /// **This function is not suitable for strings that do not contain an executable name.**
    ///
    /// ```
    /// let args = windows_args::Args::parse_cmd(r#"me.exe  \\\"#);
    /// assert_eq!(
    ///     args.collect::<Vec<_>>(),
    ///     vec!["me.exe".to_string(), r#"\\\"#.to_string()],
    /// );
    /// ```
    pub fn parse_cmd(input: &str) -> Self {
        Self::parse_cmd_os(input.as_ref())
            .unwrap_or_else(|NonUtf8ArgError { arg }| {
                panic!("\
valid UTF-8 became invalid after arg splitting?!
 Input: {:?}
BadArg: {:?}", input, arg);
            })
    }

    /// Parse an `OsStr` containing the complete command line.
    ///
    /// The output will always contain at least one argument (representing the executable name).
    /// If the input was empty, a placeholder name is given.
    ///
    /// **This function is not suitable for strings that do not contain an executable name.**
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use std::ffi::OsString;
    ///
    /// let args = windows_args::Args::parse_cmd_os("".as_ref())?;
    /// assert_eq!(
    ///     args.collect::<Vec<_>>(),
    ///     vec!["TEST.EXE".to_string()],
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_cmd_os(input: &OsStr) -> Result<Self, NonUtf8ArgError> {
        let inner = ArgsOs::parse_cmd(input)
            .map(|s| s.into_string())
            .collect::<Result<Vec<_>, _>>()
            .map_err(NonUtf8ArgError::new)?
            .into_iter();
        Ok(Args { inner })
    }
}

impl Iterator for Args {
    type Item = String;
    fn next(&mut self) -> Option<String> { self.inner.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize { self.inner.len() }
}

impl DoubleEndedIterator for Args {
    fn next_back(&mut self) -> Option<String> { self.inner.next_back() }
}

impl fmt::Debug for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Args")
            .field("inner", &self.inner.as_slice())
            .finish()
    }
}

impl Iterator for ArgsOs {
    type Item = OsString;
    fn next(&mut self) -> Option<OsString> { self.inner.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

impl ExactSizeIterator for ArgsOs {
    fn len(&self) -> usize { self.inner.len() }
}

impl DoubleEndedIterator for ArgsOs {
    fn next_back(&mut self) -> Option<OsString> { self.inner.next_back() }
}

impl fmt::Debug for ArgsOs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArgsOs")
            .field("inner", &self.inner.inner_debug())
            .finish()
    }
}

/// Error type returned by [`Args::parse_cmd_os`] when one of the arguments is not UTF-8.
#[derive(Debug, Clone)]
pub struct NonUtf8ArgError { arg: OsString }

impl NonUtf8ArgError {
    fn new(arg: OsString) -> Self { NonUtf8ArgError { arg } }
}

impl fmt::Display for NonUtf8ArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "non-utf8 argument: {:?}", self.arg)
    }
}

impl std::error::Error for NonUtf8ArgError { }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special_traits() {
        assert_eq!(Args::parse_cmd("a b").next_back(), Some("b".into()));
        assert_eq!(Args::parse_cmd_os("a b".as_ref()).unwrap().next_back(), Some("b".into()));
        assert_eq!(ArgsOs::parse_cmd("a b".as_ref()).next_back(), Some("b".into()));

        assert_eq!(Args::parse_cmd("a b").len(), 2);
        assert_eq!(Args::parse_cmd_os("a b".as_ref()).unwrap().len(), 2);
        assert_eq!(ArgsOs::parse_cmd("a b".as_ref()).len(), 2);
    }
}
