# MOTE v0.14.2 delivery receipt

## Published release

- [Public release](https://github.com/danny-dis/MOTE/releases/tag/v0.14.2),
  release ID `430558993`; verified non-draft, non-prerelease and GitHub's latest.
- Annotated tag `v0.14.2` resolves to
  `49e1d2515fa7950e13ad9d53ee4abc36281c637c` on `production-ready`.
- Maintenance commit: `90dc159`; reviewed Windows Node 20 test-launcher fix:
  `49e1d25`. The latter changes test discovery, not production behavior.
- [Candidate CI](https://github.com/danny-dis/MOTE/actions/runs/41304158443):
  all four jobs passed, including Windows, Linux, macOS and dependency audit.
- [Release workflow](https://github.com/danny-dis/MOTE/actions/runs/41304609732):
  all nine jobs passed, including repeated CI, all three platform archives,
  SDK tests against each extracted bridge, asset verification and publication.
- Python 3.10, Node 20 and Go 1.22 were exercised by cross-platform CI.
  The newer runtime used by GitHub Actions itself is separate from SDK runtimes.

## Download verification

All three public archives and `SHA256SUMS` were downloaded. Archive file sets
matched the release allowlist; every archive digest matched `SHA256SUMS`:

```text
17757d48e3e9c3e267d63091fc189dcbb3a7d45837ccb1e59b83203777359f27  mote-v0.14.2-x86_64-unknown-linux-gnu.tar.gz
2aaff1b1d71d6d2e4f2389823b56a7b3a9099c676804dbe0708f102fc7f60909  mote-v0.14.2-x86_64-pc-windows-msvc.zip
ed0d88efc20ed860c139c2a8e595861238cda7ebbc818346ef1263d4eec0690d  mote-v0.14.2-aarch64-apple-darwin.tar.gz
```

The downloaded Windows CLI returned `mote 0.14.2`; `--help` exited successfully.
Local SDK reruns using the downloaded Windows bridge passed: Python 25 tests,
TypeScript 14 tests with no skips on Node 20.20.2, and the normal Go test command.
The release workflow separately verified all SDKs on every platform archive.
All 13 release/documentation tooling tests and TypeScript installed-tarball
exports/declaration checks passed before tagging. Independent review approved
the final launcher repair with no required fixes.

## Website: prepared, not deployed

The source page has the v0.14.2 badge and matching release-notes link. Local
browser checks verified desktop and 390px mobile layout without horizontal
overflow, Go-tab selection and panel visibility, and no captured console
warnings/errors or page errors during those checks. These are local preview
receipts, not evidence of a production deployment or a full accessibility audit.

The intended production project is `mote-runtime` in `kallmedis-projects`, serving
<https://mote-runtime.vercel.app>. The production page still advertises v0.13.1.
The CLI is logged out; device sign-in requires owner approval. No replacement
project, temporary deployment or production deployment was created. Account
approval, production deployment and public asset readback remain required.

## Preservation and rollback

- `agents-checklist.md` was byte-preserved and excluded from commits.
- No other branches/worktrees, package registries or live MOTE installations
  were modified by this release.
- Retain [v0.14.1](https://github.com/danny-dis/MOTE/releases/tag/v0.14.1) as the
  binary rollback target. Preserve workspaces/configuration when reinstalling;
  no data migration was introduced. Never move a published tag or overwrite
  published archives to hide a regression.
- Durable local evidence, downloaded archives and detailed CI logs are under
  `C:/Users/pc/Documents/projects/mote-release-evidence/v0.14.2-publication`.

The [release checklist](todo.md) leaves the Vercel gate explicitly open.
