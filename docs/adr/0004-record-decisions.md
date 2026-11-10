# ADR-0004: Record architecture decisions

Status: accepted
Date: YYYY-MM-DD
Version: v1

## Context
Decisions get lost in chats and commits. Agents and humans need one trail.

## Options considered
1. Chat or commit messages only: lost, unsearchable.
2. ADR files in `docs/adr/`: versioned with code, reviewable.

## Decision
Use ADR files in `docs/adr/` per the template.

## Consequences
- Positive: traceable decisions, linkable from specs and constitution.
- Negative: small upkeep cost.
- Follow-up: link ADRs from `docs/README.md`.
