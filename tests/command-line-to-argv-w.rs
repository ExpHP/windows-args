#![cfg(windows)]

use std::collections::VecDeque;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::slice;
use std::iter;
use std::ptr;

fn new_parser(lp_cmd_line: &[u16]) -> VecDeque<OsString> {
    windows_args::ArgsOs::parse_cmd(&OsString::from_wide(lp_cmd_line)).collect()
}

unsafe fn old_parser(lp_cmd_line: &[u16]) -> VecDeque<OsString> {
    let mut ret_val = VecDeque::new();
    let mut num_args = 0;
    let parts = CommandLineToArgvW(lp_cmd_line.as_ptr(), &mut num_args);
    if parts.is_null() {
        return ret_val;
    }
    for i in 0..(num_args as isize) {
        let mut len = 0;
        let mut part = *parts.offset(i);
        while *part != 0 { part = part.offset(1); len += 1 };
        let os_string = OsString::from_wide(slice::from_raw_parts(*parts.offset(i), len));
        ret_val.push_back(os_string);
    }
    LocalFree(parts);
    ret_val
}

#[link(name="Shell32")]
extern "system" {
    fn CommandLineToArgvW(lpCmdLine: *const u16, pNumArgs: *mut u32) -> *mut *mut u16;
}
#[link(name="Kernel32")]
extern "system" {
    fn LocalFree(pNumArgs: *mut *mut u16);
}

fn test_chars() -> impl Iterator<Item=u16> {
    // ASCII, which encompasses all control characters for the algorithm
    (0..128)
        .chain(std::iter::once(0xff)) // something non-ascii
        .chain(std::iter::once(0xdaaa)) // a high surrogate
        .chain(std::iter::once(0xdeee)) // a low surrogate
}

#[test]
fn command_line_to_argv_w_equivalence() {
    // Test with no executable at the beginning
    for a in test_chars() {
        println!("{:x}", a);
        for b in test_chars() {
            for c in test_chars() {
                for d in test_chars() {
                    let ucs_2: [u16; 5] = [a, b, c, d, 0];
                    unsafe {
                        let new_result = new_parser(&ucs_2);
                        let old_result = old_parser(&ucs_2);
                        if old_result != new_result {
                            println!("ucs_2={:?}", ucs_2);
                        }
                        assert_eq!(old_result, new_result);
                    }
                }
            }
        }
    }
    // Test with an executable at the beginning
    for a in test_chars() {
        println!("{:x}", a);
        for b in test_chars() {
            for c in test_chars() {
                for d in test_chars() {
                    let ucs_2: [u16; 7] = ['a' as u16, ' ' as u16, a, b, c, d, 0];
                    unsafe {
                        let new_result = new_parser(&ucs_2);
                        let old_result = old_parser(&ucs_2);
                        if old_result != new_result {
                            println!("ucs_2={:?}", ucs_2);
                        }
                        assert_eq!(old_result, new_result);
                    }
                }
            }
        }
    }
}
