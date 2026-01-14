# Use PowerShell on Windows
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Check licenses only
license:
    cargo deny check licenses

# Run all cargo-deny checks (advisories, bans, licenses, sources)
deny:
    cargo deny check
