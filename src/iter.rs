#[cfg(windows)]
use std::ffi::{OsStr, OsString};
use wtf8::{Wtf8Buf};

use crate::{Args, Command};
#[cfg(windows)]
use crate::{ArgsOs, CommandOs};
use crate::{expect_still_utf8_own, expect_still_utf8_ref};

/// Type returned by [`IntoIterator`] for [`Args`].
#[derive(Debug, Clone)]
pub struct IntoIter {
    inner: std::iter::Chain<
        std::option::IntoIter<Wtf8Buf>,
        std::vec::IntoIter<Wtf8Buf>,
    >,
}

/// Type returned by [`Args::iter`].
#[derive(Debug, Clone)]
pub struct Iter<'a> {
    inner: std::iter::Chain<
        std::option::IntoIter<&'a str>,
        MapAsStr<std::slice::Iter<'a, Wtf8Buf>>,
    >,
}

/// Type returned by [`IntoIterator`] for [`ArgsOs`].
#[cfg(windows)]
#[derive(Debug, Clone)]
pub struct IntoIterOs {
    inner: std::iter::Chain<
        std::option::IntoIter<OsString>,
        std::vec::IntoIter<OsString>,
    >,
}

/// Type returned by [`ArgsOs::iter`].
#[cfg(windows)]
#[derive(Debug, Clone)]
pub struct IterOs<'a> {
    inner: std::iter::Chain<
        std::option::IntoIter<&'a OsString>,
        std::slice::Iter<'a, OsString>,
    >,
}

impl IntoIter {
    pub(crate) fn from_args(args: Args) -> Self {
        IntoIter { inner: None.into_iter().chain(args.inner.vec) }
    }

    pub(crate) fn from_cmd(cmd: Command) -> Self {
        IntoIter { inner: Some(Wtf8Buf::from_string(cmd.exe)).into_iter().chain(cmd.args.inner.vec) }
    }
}

impl<'a> Iter<'a> {
    pub(crate) fn from_args(args: &'a Args) -> Self {
        Iter { inner: None.into_iter().chain(MapAsStr(args.inner.vec.iter())) }
    }

    pub(crate) fn from_cmd(cmd: &'a Command) -> Self {
        Iter { inner: Some(&cmd.exe[..]).into_iter().chain(MapAsStr(cmd.args.inner.vec.iter())) }
    }
}

#[cfg(windows)]
impl IntoIterOs {
    pub(crate) fn from_args(args: ArgsOs) -> Self {
        IntoIterOs { inner: None.into_iter().chain(args.inner.vec) }
    }

    pub(crate) fn from_cmd(cmd: CommandOs) -> Self {
        IntoIterOs { inner: Some(cmd.exe).into_iter().chain(cmd.args.inner.vec) }
    }
}

#[cfg(windows)]
impl<'a> IterOs<'a> {
    pub(crate) fn from_args(args: &'a ArgsOs) -> Self {
        IterOs { inner: None.into_iter().chain(args.inner.vec.iter()) }
    }

    pub(crate) fn from_cmd(cmd: &'a CommandOs) -> Self {
        IterOs { inner: Some(&cmd.exe).into_iter().chain(cmd.args.inner.vec.iter()) }
    }
}

impl Iterator for IntoIter {
    type Item = String;
    fn next(&mut self) -> Option<String> { self.inner.next().map(expect_still_utf8_own) }
    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> { self.inner.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

#[cfg(windows)]
impl Iterator for IntoIterOs {
    type Item = OsString;
    fn next(&mut self) -> Option<OsString> { self.inner.next() }
    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

#[cfg(windows)]
impl<'a> Iterator for IterOs<'a> {
    type Item = &'a OsStr;
    fn next(&mut self) -> Option<&'a OsStr> { self.inner.next().map(|s| &s[..]) }
    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<String> { self.inner.next_back().map(expect_still_utf8_own) }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<&'a str> { self.inner.next_back() }
}

#[cfg(windows)]
impl DoubleEndedIterator for IntoIterOs {
    fn next_back(&mut self) -> Option<OsString> { self.inner.next_back() }
}

#[cfg(windows)]
impl<'a> DoubleEndedIterator for IterOs<'a> {
    fn next_back(&mut self) -> Option<&'a OsStr> { self.inner.next_back().map(|s| &s[..]) }
}

// equivalent to `.map(|s: &Wtf8Buf| expect_still_utf8_ref(s))`
#[derive(Debug, Clone)]
struct MapAsStr<I>(I);

impl<'a, I: Iterator<Item=&'a Wtf8Buf>> Iterator for MapAsStr<I> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> { self.0.next().map(|s| expect_still_utf8_ref(s)) }
    fn size_hint(&self) -> (usize, Option<usize>) { self.0.size_hint() }
}

impl<'a, I: DoubleEndedIterator<Item=&'a Wtf8Buf>> DoubleEndedIterator for MapAsStr<I> {
    fn next_back(&mut self) -> Option<&'a str> { self.0.next_back().map(|s| expect_still_utf8_ref(s)) }
}

