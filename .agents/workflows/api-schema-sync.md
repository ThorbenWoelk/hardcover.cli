---
description: How to sync and validate the Hardcover API schema
---
1.  **Check for changes**: Run the GitHub Action manually or wait for the daily run. If a drift is detected, an Issue is created.
2.  **Sync locally**:
    ```bash
    curl -s https://raw.githubusercontent.com/hardcoverapp/hardcover-docs/main/schema.graphql -o data/schema/schema.graphql
    ```
3.  **Validate**: Run `graphql-inspector diff` or `validate` manually (if installed via bun/npm):
    ```bash
    graphql-inspector diff data/schema/schema.graphql schema_remote.graphql
    ```
4.  **Confirm**: Commit the new schema to `main` to acknowledge the changes.
