# Four Reference Systems — MOTE Integration Mining

Status: architecture input for MOTE
Date: 2026-10-05

## Boundary
MOTE stays deliberately small. The four reference systems are mined into minimal reusable primitives, not imported as large subsystems.

## What MOTE should become
Provide a tiny, deterministic execution kernel around these primitives:

Capability — what the application explicitly permits.
Action — a proposed tool operation.
Candidate — an alternative action or model-produced plan.
Verifier — a deterministic or external check.
Retry/Repair — bounded recovery after a failure.
Strategy — a small policy describing how many candidates/checks may be used.
Worker — a runtime boundary for the application-owned execution target.
Event — structured lifecycle/result evidence.

## Inferstep ATLAS → bounded inference primitives
MOTE should expose optional candidate generation, verifier calls and bounded repair hooks without embedding an inference provider.

A minimal loop can be:
action -> execute -> verify -> accept OR repair

For callers that want more depth, permit a strategy configuration such as direct, candidate-set, candidate-plus-verifier or candidate-plus-repair.

MOTE must remain usable when all advanced features are disabled.

## iamvikshan Atlas → capability discipline
Before execution, validate intent/scope and capability grants.

Capabilities must be explicit, opt-in and checked for every action. Registration is not permission.

Support a pre-action hook and post-action hook so applications can implement their own intent gates and reviewers.

## ATLAS·OS → declarative run state
Do not implement a large workflow engine. Represent only the local run state needed to choose the next primitive:
READY, RUNNING, VERIFYING, REPAIRING, SUCCEEDED, FAILED, CANCELLED, BLOCKED, PARTIAL.

Keep state transitions deterministic and inspectable.

## Pacifio Atlas → lightweight provenance
Emit append-only JSON events with run ID, action ID, worker, tool, capability, inputs hash, output/artifact reference, verification result, retries, timestamps and final state.

Allow a caller to persist those events elsewhere. MOTE does not own long-term memory.

## Minimal API direction
run(request)
propose(action)
execute(action)
verify(result)
repair(failure)
cancel(run_id)
events(run_id)

Keep Python/TypeScript SDKs thin adapters over the same Rust contract.

## Security
Capabilities are authorization metadata, not an OS sandbox. Preserve the current limitation and document that real isolation belongs to the host/container/VM.

Never allow a model to grant itself capabilities or change verifier policy.

## Tests
Capability enforcement, deterministic state transitions, verifier failure, bounded retries, cancellation, event completeness, malformed model output and host-callback failure.

## Non-goals
No GUI, database, long-term memory, scheduler, browser, model server, multi-user control plane or large orchestration layer.

## Result
MOTE extracts the smallest useful kernel from the four systems: capability-gated actions, optional candidate/verification loops, deterministic run state and durable event emission.