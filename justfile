# Use PowerShell on Windows
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

run *args:
    cargo run --release -- {{ args }}

# Run all checks (cargo check, clippy, fmt, test, deny)
check: _check _clippy _fmt _test deny

# Check licenses only
license:
    cargo deny check licenses

# Run all cargo-deny checks (advisories, bans, licenses, sources)
deny:
    cargo deny check

[private]
_check:
    cargo check

[private]
_clippy:
    cargo clippy

[private]
_fmt:
    cargo fmt --check

[private]
_test:
    cargo test
