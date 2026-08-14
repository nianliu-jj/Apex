---
{
  "schema": "apex.verification.v1",
  "feature": "example-feature",
  "generation": 1,
  "status": "awaiting_confirmation",
  "created_at": "2026-08-14T00:00:00+08:00",
  "updated_at": "2026-08-14T00:00:00+08:00",
  "trace_id": "00000000000000000000000000000000",
  "requirements_hash": "blake3:0000000000000000000000000000000000000000000000000000000000000000",
  "design_hash": "blake3:0000000000000000000000000000000000000000000000000000000000000000",
  "tasks_hash": "blake3:0000000000000000000000000000000000000000000000000000000000000000",
  "verified_at": "2026-08-14T00:00:00+08:00",
  "acceptance_mode": "user"
}
---

# Verification: example-feature

## Summary

State the verified generation, result, residual risk, and acceptance decision.

## Evidence

| VAL | RQ / AC | Command or review | Result | Evidence path |
|---|---|---|---|---|
| VAL-000 | RQ-000 / AC-000 | Replace with the exact verification | pending | `docs/evidence/...` |

## Negative and boundary cases

Record failure, cancellation, duplicate, timeout, permission, and limit cases as applicable.

## Quality gates

- [ ] Format check passed.
- [ ] Lint check passed with warnings denied.
- [ ] Relevant unit, integration, property, and contract tests passed.
- [ ] Dependency, schema, protocol, and documentation drift checks passed.
- [ ] No ignored tests or untracked exceptions remain.

## Acceptance

Record user or automatic acceptance, approver identity, timestamp, and evidence hash.
