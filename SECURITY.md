# Security policy

dex-parser-analyzer parses untrusted input: the GUI and the CLI accept any file a user picks. A crash, hang or excessive allocation on a crafted DEX or APK is a security bug.

## Reporting

Report privately through GitHub's private vulnerability reporting: open the repository's **Security** tab, then **Report a vulnerability** (https://github.com/dhilipsiva/dex-parser-analyzer/security/advisories/new). Please include the input file, or a script that generates it, and the command or page that misbehaves.

Please do not open a public issue for these reports.

## What to expect

This is a solo-maintained project, so these are targets rather than guarantees:

- An acknowledgement within 7 days.
- A fix or a mitigation plan within 30 days for a confirmed issue, released as a patch version with a CHANGELOG entry and, where it applies, a RUSTSEC advisory.
- Credit in the release notes, unless you prefer otherwise.

## Supported versions

Until 1.0, only the latest release receives fixes.
