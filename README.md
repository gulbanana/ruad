Prerequisites
=============
* Rust: https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe

Environment
===========
Recommended IDE: Visual Studio Code with these extensions.
* https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer 
* https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml

Behind a corporate proxy, add this to ~/.cargo/config.toml:
```toml
[http]
check-revoke=false
```

Development
===========
Build service binary: `cargo build --release`
Test install: `target\release\ruad.exe`, or `cargo run /install` if your command prompt is elevated. 
Test execution: `cargo run /runOnce`.