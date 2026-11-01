# Release plan: MOTE v0.14.2

## Status and scope

v0.14.2 is published as the latest GitHub release at `49e1d25`, with green
cross-platform CI and verified downloaded archives. The updated website is
prepared and locally checked; Vercel production deployment still requires
account sign-in. See the [delivery receipt](release-0.14.2.md) for exact URLs,
checksums, verification results and the remaining gate. The local evidence below
is the historical preparation record, not the final delivery receipt.

Maintenance release execution is authorized: commit and push the reviewed work,
run CI/CD for v0.14.2, publish only after the required gates pass, and deploy the
updated static Vercel page. The package candidate is now 0.14.2. Package-registry
publication and upgrades of other running installations are outside scope.
The authoritative release checklist is [todo.md](todo.md).

At release preparation, the latest published release was v0.14.1. The local `production-ready` checkout was
fast-forwarded to upstream `72ac388` (the existing Go SDK website update), with all
local repairs and the unrelated agent checklist byte-preserved. CI and release
acceptance must identify the exact candidate commit; earlier upstream CI does not
certify the new candidate. The website update is prepared in source but must not
be deployed to production until binary publication is verified.

## What this patch should contain

- One action-to-capability mapping used by permission checks and tool budgets.
- Shared file-read/directory-list retry and error orchestration.
- One authentication environment lookup used by three provider adapters.
- Continuous Go stderr drainage without unused retention, and direct management
  of Python's single reader thread.
- Regressions for capability grants, budgets, authentication and actual runtime
  guard failure reasons.
- Reliable TypeScript fixture paths and terminal flushing, exact protocol/error
  assertions, and real callback-redaction checks.
- Startup-aware SDK test design: general setup gets its own allowance, explicit
  short timeout regressions remain, and Go fixture compilation is bounded.
- Changelog and stderr-handling documentation that match the code.

No new features, runtime dependencies, public API changes, protocol changes,
permission relaxations or production timeout changes are intended. Keep bridge
protocol v1 and the independent SDK package versions unchanged. Do not add a
scheduler, service, plug-in framework, browser layer or speculative extension point.

## Current local evidence

- Rust: 60 tests and 3 starter tests passed; both examples completed. The custom
  tool produced `HELLO`. Formatting and Clippy with warnings denied passed.
- Python SDK: 25 tests passed, including the real bridge.
- TypeScript SDK: the normal command passed 14 tests, including the real bridge;
  the client suite also passed from the repository root. Installed-tarball exports
  and consumer declarations passed.
- Go SDK: the normal runner passed 19 top-level tests and 13 subtests, including
  the real bridge and stderr flood. The repaired cooperative test passed three
  consecutive runs, as did the startup-aware malformed-frame regression. Vet,
  formatting and Go 1.22 language/API checks passed.
- Release/documentation tooling: 12 tests passed, including checks of links in
  this plan and its checklist and the new SDK-minimum CI guard.
- Disposable TypeScript mutations proved that the regressions detect leaked
  callback errors and incorrect error categories. The old denial assertion
  accepted the wrong category; the corrected assertion rejects it. Both checks
  passed after the disposable SDK copies were restored byte-for-byte.
- Independent source reviews approved the core refactor and corrected SDK tests.
  Both required test-review findings were resolved and the successor source was
  approved; later source changes need matching review and verification evidence.

These checks used this Windows host, including Go 1.25.5, Node 26 and Python
3.14.7. Go 1.22 language/API checks are not an actual Go 1.22 runtime run. The
current candidate still needs the cross-platform and release-artifact gates below.

## Measured performance evidence

A standalone, standard-library-only benchmark drained 200,000 bytes per iteration
through a reader-only stream adapter, for 20 iterations on the local i5-2540M:

- Prior retention loop: 548,928 allocated bytes and 9 allocations per operation.
- Discarding loop: 488 allocated bytes and 2 allocations per operation.

This isolates unused stderr retention/copying. It does not measure subprocess
startup, real pipe throughput, model latency or whole-agent speed. Do not market
these figures as an end-to-end speedup.

## Release sequence

1. Freeze and independently review the exact source/test candidate; save only the
   authorized changes and keep unrelated local work out of the release.
2. Prepare the 0.14.2 metadata cohort without unrelated dependency updates.
3. Qualify that exact candidate locally and through cross-platform CI.
4. After explicit publication authorization, create a new annotated tag and let
   the existing release workflow build, extract, test, checksum and publish it.
5. Verify the public artifacts, then update the website's latest-release label.

Detailed acceptance criteria and commands belong in [todo.md](todo.md), not in a
second competing checklist. Follow [RELEASING.md](../docs/RELEASING.md) for the
existing workflow rather than adding another release system.

## Version consumers and compatibility

At release preparation, change the root `Cargo.toml`, regenerate the root and
starter `Cargo.lock` files with Cargo, and verify that only the local `mote-agent`
version changes. Update the current root-version inventory in
`THIRD_PARTY_NOTICES.md`, current release-command examples and the changelog's new
`## 0.14.2` section. Preserve historical changelog entries, historical fixtures,
and the official 0.14.1 Go SDK specification. The website badge must not advertise
0.14.2 as published before publication is verified.

Verify the declared minimum SDK environments (Python 3.10, Node 20, Go 1.22) as
well as the existing CI environments. Keep unsigned/not-notarized binary warnings,
source-installed SDK guidance, capability boundaries and synchronous callback
cancellation limits explicit.

## Hold and rollback conditions

Hold publication for any failing suite, unreviewed source drift, unexpected
lockfile change, dependency-audit blocker, missing platform artifact, checksum
mismatch or disagreement between the tag and release commit. A successful wrapper
exit is not sufficient if an earlier command or individual suite failed.

No data migration is planned. If a published patch regresses behavior, retain user
workspaces/configuration and reinstall the verified v0.14.1 binaries with matching
SDK source. Publish a new corrective patch when qualified; never move a published
tag, overwrite an old release or delete evidence of a failed release.
