# `windows-args` [![Appveyor](https://ci.appveyor.com/api/projects/status/github/ExpHP/windows-args)](https://ci.appveyor.com/project/ExpHP/windows-args) [![Crates.io](https://img.shields.io/crates/v/windows-args.svg)](https://crates.io/crates/windows-args) [![Docs](https://docs.rs/windows-args/badge.svg)](https://docs.rs/windows-args)

A command-line argument parser for Windows, copied almost wholesale from the rust standard library.

```toml
[dependencies]
windows-args = "0.1"
```

```rust
use windows_args::Args;

// for a complete command line, with executable
for arg in Args::parse_cmd(r#"foobar.exe to "C:\Program Files\Hi.txt" now"#) {
    println!("{}", arg);
}

// for just args, without an executable
for arg in Args::parse_args(r#"to "C:\Program Files\Hi.txt" now"#) {
    println!("{}", arg);
}
```

