# hc - Hardcover CLI

A Rust CLI for interacting with the [Hardcover.app](https://hardcover.app) GraphQL API. Manage your books, lists, reading goals, and more from the terminal.

## Setup

1. Install:

```sh
cargo install --path .
```

This places the `hc` binary in `~/.cargo/bin/`, so you can run it from anywhere.

2. Get your API token from [hardcover.app/account/api](https://hardcover.app/account/api)

3. Log in:

```sh
hc login
```

This saves your token to `~/.config/hc/config.toml`.

Alternatively, set the `HARDCOVER_API_KEY` environment variable (takes precedence over the config file):

```sh
export HARDCOVER_API_KEY="Bearer <your-token>"
```

To remove stored credentials:

```sh
hc logout
```

## Commands

### Profile

```sh
hc me                              # Show your profile
```

### Search

```sh
hc search "Dune"                   # Search for books (default: 10 results)
hc search "Dune" -l 5             # Limit results
hc search "Dune" -l 5 -p 2       # Page 2 of results
```

### Book Details

```sh
hc book 312460                     # By ID
hc book dune                       # By slug
```

### Library

```sh
hc my-books                        # List your books (default: 20)
hc my-books -s read               # Filter by status
hc my-books -l 50 -o 20           # Pagination: 50 results, skip first 20
hc my-books --all                  # Fetch ALL books (auto-paginates)
hc my-books --all -s wtr          # All "want to read" books
```

Status values: `want-to-read`/`wtr`, `reading`/`cr`, `read`/`r`, `paused`/`p`, `dnf`, `ignored`/`i`

### Managing Books

```sh
hc set-status 312460 wtr           # Add book / update status
hc set-status 312460 read --rating 4.5 --notes "Great book" --owned true --review "Highly recommended"
hc set-status 312460 reading --edition-id 123 --privacy 1

hc rate 312460 4.5                 # Rate a book (0.5-5.0)

hc update 312460 --status read --rating 5 --last-read 2026-03-15 --owned true
hc update 312460 --notes "Re-read" --started 2026-01-01 --recommended-for "Bob"

hc remove-book 312460              # Remove from library
```

`update` and `set-status` support: `--status`, `--rating`, `--notes`, `--date-added`, `--last-read`, `--started`, `--edition-id`, `--privacy`, `--recommended-by`, `--recommended-for`, `--owned`, `--review`, `--spoilers`
 
### Reading Progress

```sh
hc reads 312460                    # Show read entries
hc read-add 312460 --started 2026-03-01 --finished 2026-03-15
hc read-add 312460 --progress 0.5 --pages 150   # Track progress
hc read-update 789 --progress 0.8  # Update progress
hc read-delete 789                 # Delete entry
```

### Reading Journals

```sh
hc journals                        # Show all journal entries
hc journals -b 312460              # Filter by book
hc journal-add 312460 "note" -e "Started re-reading"
hc journal-update 456 -e "Updated note"
hc journal-delete 456              # Delete entry
```

```sh
hc lists                           # Show your lists
hc list 371666                     # Show list details with books

hc list-create "Sci-Fi"            # Create a list
hc list-create "Sci-Fi" -d "Best sci-fi" --ranked true --privacy 1

hc list-update 371666 -n "New Name" -d "New description"

hc list-add 371666 312460          # Add book to list
hc list-add 371666 312460 --position 1 --edition-id 456

hc list-remove 12345               # Remove entry (use list_book_id from list details)
hc list-delete 371666              # Delete a list
```

### Discovery and Social

```sh
hc goals                           # Show your reading goals
hc trending                        # Trending books (last 7 days)
hc trending -l 20 -d 30           # Top 20 trending over 30 days
hc trending -o 10                  # Skip first 10
hc feed                            # Activity feed (default: 20)
hc feed -l 50 -o 20               # Pagination
hc character "harry-potter"        # Show character details
hc tags                            # List all tags
hc tags --category 1               # List tags by category (e.g., Genres)
hc tag 123456 "Fantasy" "Epic"     # Tag a UserBook (default)
hc tag 312460 -e Book "Sci-Fi"     # Tag a Book
hc follow 123456 -e User           # Follow a user
hc unfollow 312460 -e Book         # Unfollow a book
hc platforms                       # List all reading platforms
hc formats                         # List all reading formats
hc publishers                      # List publishers
hc notifications                   # Show your recent activity alerts
hc prompts                         # List community prompts
```

### Editions and ISBN

```sh
hc editions 312460                 # List editions for a book
hc isbn 9780441172719             # Show edition details by ISBN
hc book-create "New Book"         # Create a new book entry in the database
```

## Development

### API Schema Synchronization

This project tracks the [Hardcover API Schema](https://github.com/hardcoverapp/hardcover-docs/) to detect potential breaking changes.

- **Automated Check**: A GitHub Action runs daily to compare our local `data/schema/schema.graphql` with the upstream version. If a drift is detected, it opens a GitHub Issue with a detailed `graphql-inspector` report.
- **Manual Sync**: To manually update the local schema and acknowledge upstream changes:
  ```bash
  curl -s https://raw.githubusercontent.com/hardcoverapp/hardcover-docs/main/schema.graphql -o data/schema/schema.graphql
  ```
- **Validation**: When the schema changes, ensure all internal queries in `src/client.rs` are still compatible.

## Collaboration

We follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) standard for all commit messages. This helps in maintaining a clean and automated changelog.

### Commit Types:
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code (white-space, formatting, etc.)
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `chore`: Changes to the build process or auxiliary tools and libraries

### Example:
`feat(api): add support for reading progress tracking`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
