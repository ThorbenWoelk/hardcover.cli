use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::json;

use hc::client::HardcoverClient;
use hc::display::*;

#[derive(Parser)]
#[command(name = "hc", about = "Hardcover.app CLI - manage your books")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show your profile
    Me,
    /// Search for books, authors, series, users, or lists
    Search {
        /// Search query
        query: String,
        /// Type: Book, Author, Series, User, List (default: Book)
        #[arg(short = 't', long, default_value = "Book")]
        query_type: String,
        /// Max results per page
        #[arg(short, long, default_value = "10")]
        limit: u32,
        /// Page number (1-based)
        #[arg(short, long, default_value = "1")]
        page: u32,
        /// Search by ISBN (Books only)
        #[arg(long)]
        isbn: Option<String>,
    },
    /// Show book details by ID or slug
    Book {
        /// Book ID (number) or slug (text)
        id_or_slug: String,
    },
    /// Create a new book
    BookCreate {
        /// Title
        title: String,
        /// Description
        #[arg(short, long)]
        description: Option<String>,
        /// Number of pages
        #[arg(short, long)]
        pages: Option<i32>,
        /// Release date (YYYY-MM-DD)
        #[arg(short, long)]
        date: Option<String>,
    },
    /// Show editions for a book
    Editions {
        /// Book ID
        book_id: i64,
    },
    /// Show edition details by ISBN
    Isbn {
        /// ISBN-10 or ISBN-13
        isbn: String,
    },
    /// Show author details by ID
    Author {
        /// Author ID
        author_id: i64,
    },
    /// Show series details by ID
    Series {
        /// Series ID
        series_id: i64,
    },
    /// Show a user's profile by username
    User {
        /// Username
        username: String,
    },
    /// Show character details by ID or slug
    Character {
        /// Character ID (number) or slug (text)
        id_or_slug: String,
    },
    /// List all tags
    Tags {
        /// Filter by category ID (1: Genre, 2: Mood, 4: Content Warning, 5: Pacing, 6: Character, 7: Setting, 8: Theme)
        #[arg(short, long)]
        category: Option<i32>,
        /// Max results
        #[arg(short, long, default_value = "50")]
        limit: i32,
        /// Offset
        #[arg(short, long, default_value = "0")]
        offset: i32,
    },
    /// Show your notifications
    Notifications {
        /// Max notifications
        #[arg(short, long, default_value = "20")]
        limit: i32,
        /// Offset
        #[arg(short, long, default_value = "0")]
        offset: i32,
    },
    /// List social prompts
    Prompts {
        /// Max prompts
        #[arg(short, long, default_value = "20")]
        limit: i32,
        /// Offset
        #[arg(short, long, default_value = "0")]
        offset: i32,
    },
    /// Follow an entity (User, Book, Series, Author)
    Follow {
        /// ID of entity to follow
        id: i64,
        /// Type: User, Book, Series, Author
        #[arg(short, long, default_value = "User")]
        entity_type: String,
    },
    /// Unfollow an entity
    Unfollow {
        /// ID of entity to unfollow
        id: i64,
        /// Type: User, Book, Series, Author
        #[arg(short, long, default_value = "User")]
        entity_type: String,
    },
    /// Tag an entity (Book, UserBook, etc.)
    Tag {
        /// ID of the entity to tag
        id: i64,
        /// Type of entity (Book, UserBook, ReadingJournal)
        #[arg(short, long, default_value = "UserBook")]
        entity_type: String,
        /// List of tags
        #[arg(required = true)]
        tags: Vec<String>,
    },
    /// List available formats
    Formats,
    /// List available platforms
    Platforms,
    /// List available publishers
    Publishers {
        /// Max results
        #[arg(short, long, default_value = "50")]
        limit: i32,
        /// Offset
        #[arg(short, long, default_value = "0")]
        offset: i32,
    },
    /// List your books
    MyBooks {
        /// Filter by status: want-to-read/wtr, reading/cr, read/r, paused/p, dnf, ignored/i
        #[arg(short, long)]
        status: Option<String>,
        /// Max results (ignored with --all)
        #[arg(short, long, default_value = "20")]
        limit: i32,
        /// Skip first N results
        #[arg(short, long, default_value = "0")]
        offset: i32,
        /// Fetch all books (auto-paginates)
        #[arg(short, long)]
        all: bool,
    },
    /// Set a book's reading status (adds to library if not present)
    SetStatus {
        /// Book ID
        book_id: i64,
        /// Status: want-to-read/wtr, reading/cr, read/r, paused/p, dnf, ignored/i
        status: String,
        /// Rating (0.5-5.0)
        #[arg(short, long)]
        rating: Option<f64>,
        /// Private notes
        #[arg(long)]
        notes: Option<String>,
        /// Date added (YYYY-MM-DD)
        #[arg(long)]
        date_added: Option<String>,
        /// Edition ID
        #[arg(long)]
        edition_id: Option<i64>,
        /// Privacy setting ID
        #[arg(long)]
        privacy: Option<i32>,
        /// Owned copy
        #[arg(long)]
        owned: Option<bool>,
        /// Public review
        #[arg(long)]
        review: Option<String>,
        /// Recommended for
        #[arg(long)]
        recommended_for: Option<String>,
    },
    /// Rate a book (0.5 to 5.0 in 0.5 increments)
    Rate {
        /// Book ID
        book_id: i64,
        /// Rating (0.5-5.0)
        rating: f64,
    },
    /// Update a book in your library (by book ID)
    Update {
        /// Book ID
        book_id: i64,
        /// Status: want-to-read/wtr, reading/cr, read/r, paused/p, dnf, ignored/i
        #[arg(short, long)]
        status: Option<String>,
        /// Rating (0.5-5.0)
        #[arg(short, long)]
        rating: Option<f64>,
        /// Private notes
        #[arg(long)]
        notes: Option<String>,
        /// Date added (YYYY-MM-DD)
        #[arg(long)]
        date_added: Option<String>,
        /// Last read date (YYYY-MM-DD)
        #[arg(long)]
        last_read: Option<String>,
        /// First started reading date (YYYY-MM-DD)
        #[arg(long)]
        started: Option<String>,
        /// Edition ID
        #[arg(long)]
        edition_id: Option<i64>,
        /// Privacy setting ID
        #[arg(long)]
        privacy: Option<i32>,
        /// Recommended by
        #[arg(long)]
        recommended_by: Option<String>,
        /// Recommended for
        #[arg(long)]
        recommended_for: Option<String>,
        /// Owned copy
        #[arg(long)]
        owned: Option<bool>,
        /// Public review
        #[arg(long)]
        review: Option<String>,
        /// Spoiler warning for review
        #[arg(long)]
        spoilers: Option<bool>,
    },
    /// Remove a book from your library
    RemoveBook {
        /// Book ID
        book_id: i64,
    },

    // --- Reading dates ---
    /// Show read dates for a book
    Reads {
        /// Book ID
        book_id: i64,
    },
    /// Add a read entry (start/finish dates)
    ReadAdd {
        /// Book ID
        book_id: i64,
        /// Started date (YYYY-MM-DD)
        #[arg(long)]
        started: Option<String>,
        /// Finished date (YYYY-MM-DD)
        #[arg(long)]
        finished: Option<String>,
        /// Edition ID
        #[arg(long)]
        edition_id: Option<i64>,
        /// Progress (0.0 to 1.0)
        #[arg(long)]
        progress: Option<f64>,
        /// Progress in pages
        #[arg(long)]
        pages: Option<i32>,
    },
    /// Update a read entry
    ReadUpdate {
        /// Read entry ID
        read_id: i64,
        /// Started date (YYYY-MM-DD)
        #[arg(long)]
        started: Option<String>,
        /// Finished date (YYYY-MM-DD)
        #[arg(long)]
        finished: Option<String>,
        /// Edition ID
        #[arg(long)]
        edition_id: Option<i64>,
        /// Progress (0.0 to 1.0)
        #[arg(long)]
        progress: Option<f64>,
        /// Progress in pages
        #[arg(long)]
        pages: Option<i32>,
    },
    /// Delete a read entry
    ReadDelete {
        /// Read entry ID (from `hc reads`)
        read_id: i64,
    },

    // --- Reading journals ---
    /// Show your reading journal entries
    Journals {
        /// Filter by book ID
        #[arg(short, long)]
        book_id: Option<i64>,
        /// Max entries
        #[arg(short, long, default_value = "20")]
        limit: i32,
    },
    /// Create a journal entry
    JournalAdd {
        /// Book ID
        book_id: i64,
        /// Event type (e.g. reading_update, started, finished, note)
        event: String,
        /// Journal text
        #[arg(short, long)]
        entry: Option<String>,
        /// Date (YYYY-MM-DD)
        #[arg(long)]
        date: Option<String>,
        /// Edition ID
        #[arg(long)]
        edition_id: Option<i64>,
        /// Privacy setting ID (default: 1)
        #[arg(long, default_value = "1")]
        privacy: i32,
    },
    /// Update a journal entry
    JournalUpdate {
        /// Journal entry ID
        journal_id: i64,
        /// Event type
        #[arg(short, long)]
        event: Option<String>,
        /// Journal text
        #[arg(short, long)]
        entry: Option<String>,
        /// Date (YYYY-MM-DD)
        #[arg(long)]
        date: Option<String>,
        /// Edition ID
        #[arg(long)]
        edition_id: Option<i64>,
        /// Privacy setting ID
        #[arg(long)]
        privacy: Option<i32>,
    },
    /// Delete a journal entry
    JournalDelete {
        /// Journal entry ID
        journal_id: i64,
    },

    // --- Lists ---
    /// List your lists
    Lists,
    /// Show list details
    List {
        /// List ID
        list_id: i64,
    },
    /// Create a new list
    ListCreate {
        /// List name
        name: String,
        /// Description
        #[arg(short, long)]
        description: Option<String>,
        /// Ranked list
        #[arg(long)]
        ranked: Option<bool>,
        /// Privacy setting ID
        #[arg(long)]
        privacy: Option<i32>,
    },
    /// Update a list
    ListUpdate {
        /// List ID
        list_id: i64,
        /// New name
        #[arg(short, long)]
        name: Option<String>,
        /// New description
        #[arg(short, long)]
        description: Option<String>,
        /// Ranked list
        #[arg(long)]
        ranked: Option<bool>,
        /// Privacy setting ID
        #[arg(long)]
        privacy: Option<i32>,
    },
    /// Add a book to a list
    ListAdd {
        /// List ID
        list_id: i64,
        /// Book ID
        book_id: i64,
        /// Position in list
        #[arg(long)]
        position: Option<i32>,
        /// Edition ID
        #[arg(long)]
        edition_id: Option<i64>,
    },
    /// Remove a book from a list (use list_book_id from list details)
    ListRemove {
        /// List book ID
        list_book_id: i64,
    },
    /// Delete a list
    ListDelete {
        /// List ID
        list_id: i64,
    },
    /// Follow a list
    ListFollow {
        /// List ID
        list_id: i64,
    },
    /// Unfollow a list
    ListUnfollow {
        /// List ID
        list_id: i64,
    },

    // --- Social ---
    /// List users you follow
    Following,
    /// Like something (review, journal, activity)
    Like {
        /// ID of the item
        id: i64,
        /// Type: Review, Activity, ReadingJournal, PromptAnswer
        #[arg(short = 't', long, default_value = "Activity")]
        likeable_type: String,
    },
    /// Unlike something
    Unlike {
        /// ID of the item
        id: i64,
        /// Type: Review, Activity, ReadingJournal, PromptAnswer
        #[arg(short = 't', long, default_value = "Activity")]
        likeable_type: String,
    },

    // --- Profile ---
    /// Update your profile
    ProfileUpdate {
        /// Display name
        #[arg(long)]
        name: Option<String>,
        /// Bio
        #[arg(long)]
        bio: Option<String>,
        /// Location
        #[arg(long)]
        location: Option<String>,
        /// Username
        #[arg(long)]
        username: Option<String>,
        /// Website link
        #[arg(long)]
        link: Option<String>,
    },

    // --- Discovery ---
    /// Show your reading goals
    Goals,
    /// Show trending books
    Trending {
        /// Max results
        #[arg(short, long, default_value = "10")]
        limit: i32,
        /// Offset
        #[arg(short, long, default_value = "0")]
        offset: i32,
        /// Days to look back
        #[arg(short, long, default_value = "7")]
        days: i64,
    },
    /// Show your activity feed
    Feed {
        /// Max items
        #[arg(short, long, default_value = "20")]
        limit: i32,
        /// Offset
        #[arg(short, long, default_value = "0")]
        offset: i32,
    },
}

fn build_client() -> Result<HardcoverClient> {
    dotenvy::dotenv().ok();
    let token = std::env::var("HARDCOVER_API_KEY")
        .context("HARDCOVER_API_KEY not set. Add it to .env or export it.")?;
    Ok(HardcoverClient::new(token))
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = build_client()?;

    match cli.command {
        Commands::Me => {
            let user = client.me().await?;
            print_user(&user);
        }
        Commands::Search {
            query,
            query_type,
            limit,
            page,
            isbn,
        } => {
            if let Some(isbn_val) = isbn {
                if let Some(ed) = client.edition_by_isbn(&isbn_val).await? {
                    println!("Found via ISBN:");
                    print_edition_detail(&ed);
                    return Ok(());
                } else {
                    println!("No book found for ISBN: {isbn_val}");
                    return Ok(());
                }
            }
            let results = client.search(&query, &query_type, limit, page).await?;
            print_search_results(&results, &query_type);
        }
        Commands::Book { id_or_slug } => {
            let book = if let Ok(id) = id_or_slug.parse::<i64>() {
                client.book_by_id(id).await?
            } else {
                client.book_by_slug(&id_or_slug).await?
            };
            print_book_detail(&book);
        }
        Commands::BookCreate {
            title,
            description,
            pages,
            date,
        } => {
            let book = client
                .create_book(&title, pages, date.as_deref(), description.as_deref())
                .await?;
            println!("Created book: {} (ID: {})", book["title"], book["id"]);
        }
        Commands::Editions { book_id } => {
            let editions = client.editions_by_book_id(book_id).await?;
            print_editions(&editions);
        }
        Commands::Isbn { isbn } => {
            if let Some(ed) = client.edition_by_isbn(&isbn).await? {
                print_edition_detail(&ed);
            } else {
                println!("No edition found for ISBN: {isbn}");
            }
        }
        Commands::Author { author_id } => {
            let author = client.author_by_id(author_id).await?;
            print_author(&author);
        }
        Commands::Series { series_id } => {
            let series = client.series_by_id(series_id).await?;
            print_series(&series);
        }
        Commands::User { username } => {
            let user = client.user_by_username(&username).await?;
            print_user_profile(&user);
        }
        Commands::Character { id_or_slug } => {
            let character = if let Ok(id) = id_or_slug.parse::<i64>() {
                client.character_by_id(id).await?
            } else {
                client.character_by_slug(&id_or_slug).await?
            };
            print_character(&character);
        }
        Commands::Tags {
            category,
            limit,
            offset,
        } => {
            let tags = client.all_tags(category, limit, offset).await?;
            print_tags(&tags);
        }
        Commands::Notifications { limit, offset } => {
            let notifications = client.my_notifications(limit, offset).await?;
            print_notifications(&notifications);
        }
        Commands::Prompts { limit, offset } => {
            let prompts = client.all_prompts(limit, offset).await?;
            print_prompts(&prompts);
        }
        Commands::Formats => {
            let items = client.all_formats().await?;
            print_id_name_list("Reading Formats", &items);
        }
        Commands::Platforms => {
            let items = client.all_platforms().await?;
            print_id_name_list("Platforms", &items);
        }
        Commands::Publishers { limit, offset } => {
            let items = client.all_publishers(limit, offset).await?;
            print_id_name_list("Publishers", &items);
        }
        Commands::Follow { id, entity_type } => {
            client.follow_entity(id, &entity_type).await?;
            println!("Followed {entity_type} {id}");
        }
        Commands::Unfollow { id, entity_type } => {
            client.unfollow_entity(id, &entity_type).await?;
            println!("Unfollowed {entity_type} {id}");
        }
        Commands::Tag {
            id,
            entity_type,
            tags,
        } => {
            client.upsert_tags(id, tags, &entity_type).await?;
            println!("Updated tags for {entity_type} {id}");
        }
        Commands::MyBooks {
            status,
            limit,
            offset,
            all,
        } => {
            let status_id = status.as_deref().and_then(parse_status);
            if status.is_some() && status_id.is_none() {
                anyhow::bail!(
                    "Invalid status. Use: want-to-read/wtr, reading/cr, read/r, paused/p, dnf, ignored/i"
                );
            }
            let books = if all {
                client.my_books_all(status_id).await?
            } else {
                client.my_books(status_id, limit, offset).await?
            };
            let count = books.as_array().map(|a| a.len()).unwrap_or(0);
            print_my_books(&books);
            println!("\n{count} books shown");
        }
        Commands::SetStatus {
            book_id,
            status,
            rating,
            notes,
            date_added,
            edition_id,
            privacy,
            owned,
            review,
            recommended_for,
        } => {
            let status_id = parse_status(&status).context(
                "Invalid status. Use: want-to-read/wtr, reading/cr, read/r, paused/p, dnf, ignored/i",
            )?;

            let mut obj = json!({ "status_id": status_id });
            if let Some(r) = rating {
                obj["rating"] = json!(r);
            }
            if let Some(n) = &notes {
                obj["private_notes"] = json!(n);
            }
            if let Some(d) = &date_added {
                obj["date_added"] = json!(d);
            }
            if let Some(e) = edition_id {
                obj["edition_id"] = json!(e);
            }
            if let Some(p) = privacy {
                obj["privacy_setting_id"] = json!(p);
            }
            if let Some(o) = owned {
                obj["owned"] = json!(o);
            }
            if let Some(r) = &review {
                obj["review"] = json!(r);
            }
            if let Some(rf) = &recommended_for {
                obj["recommended_for"] = json!(rf);
            }

            if let Some(existing) = client.find_user_book_for_book(book_id).await? {
                let ub_id = existing["id"].as_i64().unwrap();
                client.update_user_book(ub_id, obj).await?;
                println!(
                    "Updated status to \"{}\" for book {book_id}",
                    status_name(status_id)
                );
            } else {
                obj["book_id"] = json!(book_id);
                client.insert_user_book(obj).await?;
                println!(
                    "Added book {book_id} with status \"{}\"",
                    status_name(status_id)
                );
            }
        }
        Commands::Rate { book_id, rating } => {
            if !(0.5..=5.0).contains(&rating) || (rating * 2.0).fract() != 0.0 {
                anyhow::bail!("Rating must be 0.5-5.0 in 0.5 increments");
            }

            if let Some(existing) = client.find_user_book_for_book(book_id).await? {
                let ub_id = existing["id"].as_i64().unwrap();
                client
                    .update_user_book(ub_id, json!({ "rating": rating }))
                    .await?;
                println!("Rated book {book_id} as {rating:.1}");
            } else {
                anyhow::bail!(
                    "Book {book_id} is not in your library. Add it first with `hc set-status {book_id} <status>`"
                );
            }
        }
        Commands::Update {
            book_id,
            status,
            rating,
            notes,
            date_added,
            last_read,
            started,
            edition_id,
            privacy,
            recommended_by,
            recommended_for,
            owned,
            review,
            spoilers,
        } => {
            let existing = client
                .find_user_book_for_book(book_id)
                .await?
                .context(format!(
                    "Book {book_id} not in library. Use `hc set-status {book_id} <status>` first"
                ))?;
            let ub_id = existing["id"].as_i64().unwrap();

            let mut updates = json!({});
            if let Some(s) = status {
                updates["status_id"] = json!(parse_status(&s).context("Invalid status")?);
            }
            if let Some(r) = rating {
                updates["rating"] = json!(r);
            }
            if let Some(n) = notes {
                updates["private_notes"] = json!(n);
            }
            if let Some(d) = date_added {
                updates["date_added"] = json!(d);
            }
            if let Some(d) = last_read {
                updates["last_read_date"] = json!(d);
            }
            if let Some(d) = started {
                updates["first_started_reading_date"] = json!(d);
            }
            if let Some(e) = edition_id {
                updates["edition_id"] = json!(e);
            }
            if let Some(p) = privacy {
                updates["privacy_setting_id"] = json!(p);
            }
            if let Some(r) = recommended_by {
                updates["recommended_by"] = json!(r);
            }
            if let Some(r) = recommended_for {
                updates["recommended_for"] = json!(r);
            }
            if let Some(o) = owned {
                updates["owned"] = json!(o);
            }
            if let Some(r) = review {
                updates["review"] = json!(r);
            }
            if let Some(s) = spoilers {
                updates["review_has_spoilers"] = json!(s);
            }

            client.update_user_book(ub_id, updates).await?;
            println!("Updated book {book_id}");
        }
        Commands::RemoveBook { book_id } => {
            if let Some(existing) = client.find_user_book_for_book(book_id).await? {
                let ub_id = existing["id"].as_i64().unwrap();
                client.delete_user_book(ub_id).await?;
                println!("Removed book {book_id} from your library");
            } else {
                println!("Book {book_id} is not in your library");
            }
        }

        // --- Read dates ---
        Commands::Reads { book_id } => {
            let reads = client.book_reads(book_id).await?;
            print_reads(&reads, book_id);
        }
        Commands::ReadAdd {
            book_id,
            started,
            finished,
            edition_id,
            progress,
            pages,
        } => {
            let existing = client
                .find_user_book_for_book(book_id)
                .await?
                .context(format!("Book {book_id} not in library"))?;
            let ub_id = existing["id"].as_i64().unwrap();
            client
                .add_book_read(
                    ub_id,
                    started.as_deref(),
                    finished.as_deref(),
                    edition_id,
                    progress,
                    pages,
                )
                .await?;
            println!("Added read entry for book {book_id}");
        }
        Commands::ReadUpdate {
            read_id,
            started,
            finished,
            edition_id,
            progress,
            pages,
        } => {
            let mut updates = json!({});
            if let Some(s) = started {
                updates["started_at"] = json!(s);
            }
            if let Some(f) = finished {
                updates["finished_at"] = json!(f);
            }
            if let Some(e) = edition_id {
                updates["edition_id"] = json!(e);
            }
            if let Some(p) = progress {
                updates["progress"] = json!(p);
            }
            if let Some(pp) = pages {
                updates["progress_pages"] = json!(pp);
            }
            client.update_book_read(read_id, updates).await?;
            println!("Updated read entry {read_id}");
        }
        Commands::ReadDelete { read_id } => {
            client.delete_book_read(read_id).await?;
            println!("Deleted read entry {read_id}");
        }

        // --- Journals ---
        Commands::Journals { book_id, limit } => {
            let journals = client.my_journals(book_id, limit).await?;
            print_journals(&journals);
        }
        Commands::JournalAdd {
            book_id,
            event,
            entry,
            date,
            edition_id,
            privacy,
        } => {
            client
                .create_journal(
                    book_id,
                    &event,
                    entry.as_deref(),
                    date.as_deref(),
                    edition_id,
                    privacy,
                )
                .await?;
            println!("Created journal entry for book {book_id}");
        }
        Commands::JournalUpdate {
            journal_id,
            event,
            entry,
            date,
            edition_id,
            privacy,
        } => {
            let mut updates = json!({});
            if let Some(e) = event {
                updates["event"] = json!(e);
            }
            if let Some(e) = entry {
                updates["entry"] = json!(e);
            }
            if let Some(d) = date {
                updates["action_at"] = json!(d);
            }
            if let Some(eid) = edition_id {
                updates["edition_id"] = json!(eid);
            }
            if let Some(p) = privacy {
                updates["privacy_setting_id"] = json!(p);
            }
            client.update_journal(journal_id, updates).await?;
            println!("Updated journal entry {journal_id}");
        }
        Commands::JournalDelete { journal_id } => {
            client.delete_journal(journal_id).await?;
            println!("Deleted journal entry {journal_id}");
        }

        // --- Lists ---
        Commands::Lists => {
            let lists = client.my_lists().await?;
            print_lists(&lists);
        }
        Commands::List { list_id } => {
            let list = client.list_details(list_id).await?;
            print_list_detail(&list);
        }
        Commands::ListCreate {
            name,
            description,
            ranked,
            privacy,
        } => {
            let result = client
                .create_list(&name, description.as_deref(), ranked, privacy)
                .await?;
            println!("Created list: {}", result["insert_list"]["name"]);
            println!("ID: {}", result["insert_list"]["id"]);
        }
        Commands::ListUpdate {
            list_id,
            name,
            description,
            ranked,
            privacy,
        } => {
            client
                .update_list(
                    list_id,
                    name.as_deref(),
                    description.as_deref(),
                    ranked,
                    privacy,
                )
                .await?;
            println!("Updated list {list_id}");
        }
        Commands::ListAdd {
            list_id,
            book_id,
            position,
            edition_id,
        } => {
            let result = client
                .add_book_to_list(list_id, book_id, position, edition_id)
                .await?;
            let title = result["insert_list_book"]["book"]["title"]
                .as_str()
                .unwrap_or("unknown");
            println!("Added \"{title}\" to list {list_id}");
        }
        Commands::ListRemove { list_book_id } => {
            client.remove_list_book(list_book_id).await?;
            println!("Removed entry {list_book_id} from list");
        }
        Commands::ListDelete { list_id } => {
            client.delete_list(list_id).await?;
            println!("Deleted list {list_id}");
        }
        Commands::ListFollow { list_id } => {
            client.follow_list(list_id).await?;
            println!("Followed list {list_id}");
        }
        Commands::ListUnfollow { list_id } => {
            client.unfollow_list(list_id).await?;
            println!("Unfollowed list {list_id}");
        }

        // --- Social ---
        Commands::Following => {
            let following = client.my_following().await?;
            print_following(&following);
        }
        Commands::Like { id, likeable_type } => {
            client.like(id, &likeable_type).await?;
            println!("Liked {likeable_type} {id}");
        }
        Commands::Unlike { id, likeable_type } => {
            client.unlike(id, &likeable_type).await?;
            println!("Unliked {likeable_type} {id}");
        }

        // --- Profile ---
        Commands::ProfileUpdate {
            name,
            bio,
            location,
            username,
            link,
        } => {
            let mut updates = json!({});
            if let Some(n) = name {
                updates["name"] = json!(n);
            }
            if let Some(b) = bio {
                updates["bio"] = json!(b);
            }
            if let Some(l) = location {
                updates["location"] = json!(l);
            }
            if let Some(u) = username {
                updates["username"] = json!(u);
            }
            if let Some(l) = link {
                updates["link"] = json!(l);
            }
            client.update_profile(updates).await?;
            println!("Profile updated");
        }

        // --- Discovery ---
        Commands::Goals => {
            let goals = client.my_goals().await?;
            print_goals(&goals);
        }
        Commands::Trending {
            limit,
            offset,
            days,
        } => {
            let books = client.trending_books(limit, offset, days).await?;
            print_trending(&books);
        }
        Commands::Feed { limit, offset } => {
            let feed = client.activity_feed(limit, offset).await?;
            print_feed(&feed);
        }
    }

    Ok(())
}
