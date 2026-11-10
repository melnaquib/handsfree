# Testing

## Levels
- Unit: pure logic, no OS calls
- Integration: one real automation path per OS
- Manual: desktop flows in a test VM

## Run
TODO commands

## Test accounts and VMs
TODO (no real user accounts)

## Quality standards (from template README)

- Zero tolerance for defects: all commits pass the quality gate.
- 80%+ test coverage: property tests, unit tests, integration tests.
- Type safety: schemars JsonSchema with validation (PMCP path).
- Production middleware: client tracking, redaction, request IDs (PMCP path).
- Quality gate: `make quality-gate` (fmt, clippy, tests). Workflow details in [tech/pmcp-workflow.md](tech/pmcp-workflow.md).
