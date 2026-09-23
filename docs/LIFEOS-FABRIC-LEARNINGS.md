# LifeOS + Fabric Learnings for MOTE

MOTE remains a small agent substrate. It should borrow the reusable capability philosophy from Fabric and the adaptive execution discipline from LifeOS without becoming a large personal-AI framework.

## Agent capability contracts

MOTE agents should be able to declare typed capabilities with inputs, outputs, requirements, permissions, side effects and verification expectations.

## Composable skills

Small capabilities should compose into pipelines rather than requiring one monolithic agent. MOTE can provide the execution primitives while DMR-X chooses the model/worker implementation.

## Verification hooks

Agents should expose success criteria and optional evidence hooks. An agent assertion is not automatically proof.

## Adaptive execution

MOTE should remain capable of lightweight execution while allowing escalation to stronger models or external workers through DMR-X. Avoid hardcoding model names into the substrate.

## Keep it small

Do not import LifeOS's full personal operating system, dashboard, memory or daemon architecture into MOTE. MOTE is the reusable agent-building substrate.
