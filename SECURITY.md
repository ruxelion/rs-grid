# Security Policy

## Supported versions

rs-grid is in early development (pre-1.0). Security fixes are applied to the
latest `main` and the most recent release only.

## Reporting a vulnerability

**Please do not report security vulnerabilities through public GitHub issues,
discussions, or pull requests.**

Instead, use GitHub's private vulnerability reporting:

1. Go to the repository's **Security** tab.
2. Click **Report a vulnerability**.
3. Provide a description, reproduction steps, affected versions, and impact.

Alternatively, you can reach us by email at **security@ruxelion.com**.

We aim to acknowledge reports within **5 business days** and to provide a
remediation timeline after triage. Please give us a reasonable window to
release a fix before any public disclosure.

## Scope

rs-grid is a client-side rendering library compiled to WebAssembly. It does
not handle authentication, networking, or persistence on its own. Reports are
most relevant for:

- memory-safety or panics reachable from untrusted input,
- DoS via pathological data/dimensions,
- XSS or injection through cell content rendering or theming.

The example apps (`examples/`) are demos and not intended for production use.
