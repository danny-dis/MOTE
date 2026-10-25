# MOTE — Mining Open Research, OR-Agent and HEC Open Research

MOTE should remain minimal. Only the small reusable execution primitives should be absorbed.

## Adopt

### Deterministic probes

Inspired by OR-Agent environment callbacks, expose lightweight application-owned probes for capability availability, filesystem/workspace state, model health, tool availability, runtime state, resource budget and configuration.

A probe returns typed observations/evidence rather than an LLM explanation.

### Experiment limits

Use MOTE's existing run/tool/observation limits as the boundary for repeatable experiments.

### Event semantics

Make experiment/probe lifecycle events easy to consume:
~~~text
experiment.started
probe.executed
tool.executed
observation.recorded
experiment.finished
~~~

### Benchmark friendliness

MOTE should be easy to embed in a deterministic harness so Ghost Research, DMR-X or Ghost Factory can test agent runtimes without adopting a heavyweight orchestration stack.

## Do not import

Do not add a scheduler, database, memory system, research ontology or graph engine to MOTE.

MOTE remains an execution substrate whose application owns scheduling, storage, approvals and higher-level evaluation.

## Implementation target

Document and standardize a tiny Probe contract and an Experiment Observation envelope, with explicit capability scope and resource accounting.