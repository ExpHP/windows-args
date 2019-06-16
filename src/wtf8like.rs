use wtf8::{Wtf8, Wtf8Buf};

pub(crate) trait IsWtf8Slice {
    fn encode_wide(&self) -> Vec<u16>;
}

pub(crate) trait IsWtf8Buf: Sized {
    fn from_wide(wide: &[u16]) -> Self;
    fn from_str(str: &str) -> Self;
}

#[cfg(windows)]
mod windows_impls {
    use super::*;
    use std::ffi::{OsStr, OsString};
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    impl IsWtf8Slice for OsStr {
        fn encode_wide(&self) -> Vec<u16> {
            <OsStr as OsStrExt>::encode_wide(self).collect()
        }
    }

    impl IsWtf8Buf for OsString {
        fn from_wide(wide: &[u16]) -> Self {
            <OsString as OsStringExt>::from_wide(wide)
        }

        fn from_str(s: &str) -> Self {
            s.into()
        }
    }
}

impl IsWtf8Slice for Wtf8 {
    fn encode_wide(&self) -> Vec<u16> {
        self.to_ill_formed_utf16().collect()
    }
}

impl IsWtf8Buf for Wtf8Buf {
    fn from_wide(wide: &[u16]) -> Self {
        Wtf8Buf::from_ill_formed_utf16(wide)
    }

    fn from_str(s: &str) -> Self {
        Wtf8Buf::from_str(s)
    }
}
