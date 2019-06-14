//!
//! # `windows-args`
//!
//! A command-line argument parser for Windows, copied almost wholesale from the rust standard library.
//!
//! ```rust
//! let args = args::Args::parse(r#"foobar to "C:\Program Files\Hi.txt" now"#);
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
    pub fn parse(arg_str: &OsStr) -> Self {
        ArgsOs { inner: crate::args::Args::parse(arg_str) }
    }
}

impl Args {
    pub fn parse(arg_str: &str) -> Self {
        Self::parse_os(arg_str.as_ref()).unwrap()
    }

    pub fn parse_os(arg_str: &OsStr) -> Result<Self, NonUtf8Arg> {
        let inner = ArgsOs::parse(arg_str)
            .map(|s| s.into_string())
            .collect::<Result<Vec<_>, _>>()
            .map_err(NonUtf8Arg::new)?
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

#[derive(Debug, Clone)]
pub struct NonUtf8Arg { arg: OsString }

impl NonUtf8Arg {
    fn new(arg: OsString) -> Self { NonUtf8Arg { arg } }
}

impl fmt::Display for NonUtf8Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "non-utf8 argument: {:?}", self.arg)
    }
}

impl std::error::Error for NonUtf8Arg { }
