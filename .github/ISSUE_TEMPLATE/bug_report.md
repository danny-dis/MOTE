---
name: Bug report
about: Report a reproducible runtime, SDK, packaging, or documentation problem
title: "[Bug] "
labels: ''
assignees: ''
---

## What happened?
Describe expected and actual behavior.

## Reproduction
Provide the smallest command, code, and redacted manifest that demonstrates the problem.

## Environment
- MOTE version / commit:
- OS and architecture:
- CLI, Rust embedding, Python SDK, or TypeScript SDK:
- Python/Node version, if relevant:
- Provider adapter (no credentials):

## Evidence
Include exit status and relevant redacted errors. A completed state alone does not
prove output correctness; show the mismatched artifact if safe to share.

## Checks
- [ ] I removed credentials, private data, and sensitive logs.
- [ ] I checked docs/QUICKSTART.md troubleshooting and existing issues.
- [ ] This is not a private security report (see SECURITY.md).
