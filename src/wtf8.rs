#[cfg(all(not(windows), not(feature = "_debug_compile_test")))]
const NO_COMPILE_WITHOUT_WINDOWS: () = {
    // OsStr on Windows is encoded in WTF-8, whereas on Unix it can contain arbitrary bytes.
    //
    // TODO: The implementation of encode_wide probably works on `str`, since UTF-8 should
    //       be a subset of WTF-8?  Maybe we can provide *something*...
    compile_error!{"\
        The current implementation of the 'windows_args' crate depends on the representation \
        of OsStr in Windows.  Thus, this crate cannot be used on other platforms.\
    "}
};

#[cfg(all(windows, not(feature = "_debug_compile_test")))]
pub use std::os::windows::ffi::OsStrExt;

#[cfg(feature = "_debug_compile_test")]
pub use self::fake_stuff_for_unix_compilation::{OsStrExt, OsStringExt};

#[cfg(feature = "_debug_compile_test")]
mod fake_stuff_for_unix_compilation {
    use std::ffi::{OsStr, OsString};

    pub trait OsStrExt {
        fn encode_wide(&self) -> EncodeWide<'_>;
    }

    pub trait OsStringExt {
        fn from_wide(wide: &[u16]) -> Self;
    }

    impl OsStrExt for OsStr {
        fn encode_wide(&self) -> EncodeWide<'_> {
            unimplemented!("\
                Attempted to run code with features=\"_debug_compile_test\"; \
                this option only exists for 'cargo check'!\
            ");
        }
    }

    impl OsStringExt for OsString {
        fn from_wide(_: &[u16]) -> Self {
            unimplemented!("\
                Attempted to run code with features=\"_debug_compile_test\"; \
                this option only exists for 'cargo check'!\
            ");
        }
    }

    #[derive(Clone)]
    pub struct EncodeWide<'a> { _unused: &'a OsStr }

    impl Iterator for EncodeWide<'_> {
        type Item = u16;
        fn next(&mut self) -> Option<u16> { unreachable!() }
    }

}

