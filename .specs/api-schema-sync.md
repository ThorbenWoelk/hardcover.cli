# Spec: API Schema Sync

## Goal
Resolve Issue #6 by syncing the local GraphQL schema with the remote one.

## Context
The Hardcover API schema is maintained in the `hardcover-docs` repository. We need to keep a local copy to validate our queries and possibly for code generation.

## Steps
1. Fetch latest schema from `https://raw.githubusercontent.com/hardcoverapp/hardcover-docs/main/schema.graphql`.
2. Compare with `data/schema/schema.graphql`.
3. Update if needed.
4. Verify with tests.
