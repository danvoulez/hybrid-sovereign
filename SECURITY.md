# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| v6.x    | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in this project:

1. **Do not** open a public issue.
2. Email the maintainers with a description of the vulnerability, steps to reproduce, and potential impact.
3. Allow reasonable time for a fix before public disclosure.

We will acknowledge receipt and provide an initial assessment within 7 days.

## Security Model

This workspace implements a **zero-trust** architecture:

- Workers run in sandboxed environments (no syscalls, no arbitrary I/O).
- All decisions are proof-carrying and replayable.
- State is content-addressed; mutation is pointer advancement only.
- Federation does not sync mutable state; only signed pointers over proofs.

Vulnerabilities that violate these invariants (e.g., worker escape, transcript forgery, pointer overwrite) are treated as high severity.
