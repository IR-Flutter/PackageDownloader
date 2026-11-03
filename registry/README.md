# PackageDownloader Canonical Registry

This directory contains the canonical registry of development environment mirrors for PackageDownloader 2.0.

## Files

- `mirrors.json`: Canonical list of development mirrors and their ecosystem endpoints.
- `mirrors.json.sig`: Ed25519 detached digital signature over `mirrors.json`.
- `schema.json`: JSON Schema draft 2020-12 definition validating the structure of `mirrors.json`.

## Remote Registry Distribution

The application fetches updates from:
`https://raw.githubusercontent.com/IR-Flutter/PackageDownloader/main/registry/mirrors.json`

And validates against:
`https://raw.githubusercontent.com/IR-Flutter/PackageDownloader/main/registry/mirrors.json.sig`

## Security Requirements

1. **Digital Signatures:** All registry updates must be signed with the trusted project release key before publication.
2. **Anti-Rollback:** The `revision` integer must be monotonically incremented on every change. The application client rejects any update where `revision <= current_revision`.
3. **No Unverified Endpoints:** Pull Requests introducing or modifying mirrors must include proof of technical verification for each enabled provider endpoint.
