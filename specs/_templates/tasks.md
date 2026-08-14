---
{
  "schema": "apex.tasks.v1",
  "feature": "example-feature",
  "generation": 1,
  "status": "draft",
  "created_at": "2026-08-14T00:00:00+08:00",
  "updated_at": "2026-08-14T00:00:00+08:00",
  "trace_id": "00000000000000000000000000000000",
  "requirements_hash": "blake3:0000000000000000000000000000000000000000000000000000000000000000",
  "design_hash": "blake3:0000000000000000000000000000000000000000000000000000000000000000",
  "write_paths": ["path/to/owned/files/**"],
  "verification_commands": ["replace-with-deterministic-command"]
}
---

# Tasks: example-feature

## Execution unit

### EP-0000

- Goal: State the single primary deliverable.
- Requirement/AC: RQ-000 / AC-000.
- Dependencies: List completed EP evidence required before work starts.
- Write scope: Match the frontmatter `write_paths`.
- Non-scope: State what must not be changed.
- Failure case: Define at least one negative or boundary path.
- Verification: List the exact command and expected evidence.
- Rollback: Describe how this EP can be reverted independently.

## Completion checklist

- [ ] Public API and errors are documented.
- [ ] Normal, boundary, failure, cancellation, and repeat paths are covered as applicable.
- [ ] Formatting, lint, tests, and drift checks pass.
- [ ] RQ → AC → EP → VAL evidence is complete.
