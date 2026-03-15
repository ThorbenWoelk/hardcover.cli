use colored::Colorize;
use serde_json::Value;

fn get_str<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(|v| v.as_str()).unwrap_or("-")
}

fn get_num(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|v| v.as_f64())
        .map(|n| format!("{n:.1}"))
        .unwrap_or_else(|| "-".to_string())
}

fn get_int(v: &Value, key: &str) -> String {
    v.get(key)
        .map(|val| match val {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            _ => "-".to_string(),
        })
        .unwrap_or_else(|| "-".to_string())
}

fn author_names(v: &Value) -> String {
    v.get("cached_contributors")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c["author"]["name"].as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_else(|| "-".to_string())
}

pub fn status_name(id: i32) -> &'static str {
    match id {
        1 => "Want to Read",
        2 => "Currently Reading",
        3 => "Read",
        4 => "Paused",
        5 => "Did Not Finish",
        6 => "Ignored",
        _ => "Unknown",
    }
}

pub fn parse_status(s: &str) -> Option<i32> {
    match s.to_lowercase().as_str() {
        "want-to-read" | "wtr" | "1" => Some(1),
        "reading" | "currently-reading" | "cr" | "2" => Some(2),
        "read" | "r" | "3" => Some(3),
        "paused" | "p" | "4" => Some(4),
        "dnf" | "did-not-finish" | "5" => Some(5),
        "ignored" | "i" | "6" => Some(6),
        _ => None,
    }
}

fn rating_stars(rating: f64) -> String {
    let full = rating.floor() as usize;
    let half = (rating - rating.floor()) >= 0.5;
    let mut s = String::new();
    for _ in 0..full {
        s.push('*');
    }
    if half {
        s.push('~');
    }
    s
}

pub fn print_user(user: &Value) {
    println!(
        "{} {}",
        "User:".bold(),
        get_str(user, "username").cyan()
    );
    println!("{} {}", "ID:".bold(), get_int(user, "id"));
}

pub fn print_book_detail(book: &Value) {
    let title = get_str(book, "title");
    let subtitle = book
        .get("subtitle")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    println!("{}", title.bold().cyan());
    if !subtitle.is_empty() {
        println!("{}", subtitle.dimmed());
    }
    println!("{} {}", "Author:".bold(), author_names(book));
    println!("{} {}", "Year:".bold(), get_int(book, "release_year"));
    println!("{} {}", "Pages:".bold(), get_int(book, "pages"));
    println!(
        "{} {} ({} ratings, {} users)",
        "Rating:".bold(),
        get_num(book, "rating"),
        get_int(book, "ratings_count"),
        get_int(book, "users_count")
    );
    println!("{} {}", "Slug:".bold(), get_str(book, "slug"));
    println!("{} {}", "ID:".bold(), get_int(book, "id"));

    if let Some(tags) = book.get("cached_tags").and_then(|t| t.as_array()) {
        let tag_names: Vec<&str> = tags
            .iter()
            .filter_map(|t| t["tag"].as_str())
            .take(10)
            .collect();
        if !tag_names.is_empty() {
            println!("{} {}", "Tags:".bold(), tag_names.join(", "));
        }
    }

    if let Some(desc) = book.get("description").and_then(|d| d.as_str()) {
        if !desc.is_empty() {
            println!("\n{}", "Description:".bold());
            // Trim HTML tags roughly
            let clean = desc
                .replace("<br>", "\n")
                .replace("<br/>", "\n")
                .replace("<br />", "\n")
                .replace("<p>", "")
                .replace("</p>", "\n");
            let re_simple = strip_html(&clean);
            println!("{}", re_simple.dimmed());
        }
    }
}

fn strip_html(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    result
}

pub fn print_search_results(results: &Value, query_type: &str) {
    let hits = match results.get("hits").and_then(|h| h.as_array()) {
        Some(h) => h,
        None => {
            println!("No results found.");
            return;
        }
    };

    let found = results
        .get("found")
        .and_then(|f| f.as_u64())
        .unwrap_or(0);
    println!("{} results found\n", found.to_string().bold());

    for hit in hits {
        let doc = &hit["document"];
        let id = get_int(doc, "id");

        match query_type {
            "Author" => {
                let name = get_str(doc, "name");
                let books = get_int(doc, "books_count");
                println!("  {} {} ({} books)", format!("[{id}]").dimmed(), name.bold(), books);
            }
            "Series" => {
                let name = get_str(doc, "name");
                let books = get_int(doc, "books_count");
                println!("  {} {} ({} books)", format!("[{id}]").dimmed(), name.bold(), books);
            }
            "User" => {
                let username = get_str(doc, "username");
                let books = get_int(doc, "books_count");
                println!("  {} {} ({} books)", format!("[{id}]").dimmed(), username.bold(), books);
            }
            "List" => {
                let name = get_str(doc, "name");
                let books = get_int(doc, "books_count");
                println!("  {} {} ({} books)", format!("[{id}]").dimmed(), name.bold(), books);
            }
            "Character" => {
                let name = get_str(doc, "name");
                println!("  {} {}", format!("[{id}]").dimmed(), name.bold());
            }
            _ => {
                // Book (default)
                let title = get_str(doc, "title");
                let year = get_int(doc, "release_year");
                let authors = doc
                    .get("author_names")
                    .and_then(|a| a.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|a| a.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_else(|| "-".to_string());
                let rating_val = doc.get("rating").and_then(|r| r.as_f64()).unwrap_or(0.0);
                let users = get_int(doc, "users_count");
                println!(
                    "  {} {} ({}) - {} [{} {} users]",
                    format!("[{id}]").dimmed(),
                    title.bold(),
                    year,
                    authors,
                    rating_stars(rating_val),
                    users,
                );
            }
        }
    }
}

pub fn print_my_books(books: &Value) {
    let arr = match books.as_array() {
        Some(a) => a,
        None => {
            println!("No books found.");
            return;
        }
    };

    if arr.is_empty() {
        println!("No books found.");
        return;
    }

    for ub in arr {
        let book = &ub["book"];
        let title = get_str(book, "title");
        let book_id = get_int(book, "id");
        let status_id = ub
            .get("status_id")
            .and_then(|s| s.as_i64())
            .unwrap_or(0) as i32;
        let rating_val = ub.get("rating").and_then(|r| r.as_f64());
        let authors = author_names(book);
        let year = get_int(book, "release_year");

        let status_str = status_name(status_id);
        let rating_str = rating_val
            .map(|r| format!("{} {r:.1}", rating_stars(r)))
            .unwrap_or_else(|| "unrated".to_string());

        let owned = ub.get("owned").and_then(|o| o.as_bool()).unwrap_or(false);
        let owned_str = if owned { " [OWNED]".green() } else { "".normal() };

        println!(
            "  {} {} ({}) - {} | {} | {}{}",
            format!("[{book_id}]").dimmed(),
            title.bold(),
            year,
            authors,
            status_str.yellow(),
            rating_str,
            owned_str,
        );
    }
}

pub fn print_lists(lists: &Value) {
    let arr = match lists.as_array() {
        Some(a) => a,
        None => {
            println!("No lists found.");
            return;
        }
    };

    if arr.is_empty() {
        println!("No lists found.");
        return;
    }

    for list in arr {
        let id = get_int(list, "id");
        let name = get_str(list, "name");
        let count = get_int(list, "books_count");
        let public = list
            .get("public")
            .and_then(|p| p.as_bool())
            .unwrap_or(false);
        let vis = if public { "public" } else { "private" };

        println!(
            "  {} {} - {} books ({})",
            format!("[{id}]").dimmed(),
            name.bold(),
            count,
            vis
        );
    }
}

pub fn print_list_detail(list: &Value) {
    let name = get_str(list, "name");
    let desc = list
        .get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("");
    let count = get_int(list, "books_count");

    println!("{} ({} books)", name.bold().cyan(), count);
    if !desc.is_empty() {
        println!("{}", desc.dimmed());
    }
    println!();

    if let Some(books) = list.get("list_books").and_then(|lb| lb.as_array()) {
        for lb in books {
            let pos = get_int(lb, "position");
            let lb_id = get_int(lb, "id");
            let book = &lb["book"];
            let title = get_str(book, "title");
            let book_id = get_int(book, "id");
            let authors = author_names(book);

            println!(
                "  {}. {} {} - {} (list_book_id: {})",
                pos,
                format!("[{book_id}]").dimmed(),
                title.bold(),
                authors,
                lb_id,
            );
        }
    }
}

pub fn print_trending(books: &Value) {
    let arr = match books.as_array() {
        Some(a) => a,
        None => {
            println!("No trending books.");
            return;
        }
    };

    for (i, book) in arr.iter().enumerate() {
        let title = get_str(book, "title");
        let id = get_int(book, "id");
        let authors = author_names(book);
        let rating = get_num(book, "rating");
        let users = get_int(book, "users_count");

        println!(
            "  {}. {} {} - {} [rating: {}, {} users]",
            i + 1,
            format!("[{id}]").dimmed(),
            title.bold(),
            authors,
            rating,
            users,
        );
    }
}

pub fn print_goals(goals: &Value) {
    let arr = match goals.as_array() {
        Some(a) => a,
        None => {
            println!("No goals found.");
            return;
        }
    };

    if arr.is_empty() {
        println!("No goals found.");
        return;
    }

    for goal in arr {
        let id = get_int(goal, "id");
        let desc = get_str(goal, "description");
        let target = get_int(goal, "goal");
        let metric = get_str(goal, "metric");
        let start = get_str(goal, "start_date");
        let end = get_str(goal, "end_date");

        println!(
            "  {} {} - {} {} ({} to {})",
            format!("[{id}]").dimmed(),
            desc.bold(),
            target,
            metric,
            start,
            end,
        );
    }
}

pub fn print_feed(feed: &Value) {
    let arr = match feed.as_array() {
        Some(a) => a,
        None => {
            println!("No activity.");
            return;
        }
    };

    for item in arr {
        let user = get_str(&item["user"], "username");
        let action = get_str(item, "event");
        let book_title = item
            .get("book")
            .and_then(|b| b.get("title"))
            .and_then(|t| t.as_str())
            .unwrap_or("");
        let created = get_str(item, "created_at");

        println!(
            "  {} {} {} {}",
            user.cyan(),
            action,
            book_title.bold(),
            format!("({created})").dimmed()
        );
    }
}

pub fn print_author(author: &Value) {
    let name = get_str(author, "name");
    let bio = author.get("bio").and_then(|b| b.as_str()).unwrap_or("");
    let born = get_int(author, "born_year");
    let died = get_int(author, "death_year");
    let location = get_str(author, "location");
    let books = get_int(author, "books_count");
    let users = get_int(author, "users_count");

    println!("{}", name.bold().cyan());
    println!("{} {}", "ID:".bold(), get_int(author, "id"));
    if born != "-" {
        let life = if died != "-" {
            format!("{born} - {died}")
        } else {
            format!("b. {born}")
        };
        println!("{} {}", "Life:".bold(), life);
    }
    if location != "-" {
        println!("{} {}", "Location:".bold(), location);
    }
    println!("{} {} books, {} users", "Stats:".bold(), books, users);
    if !bio.is_empty() {
        println!("\n{}", strip_html(bio).dimmed());
    }
}

pub fn print_series(series: &Value) {
    let name = get_str(series, "name");
    let books_count = get_int(series, "books_count");
    let completed = series
        .get("is_completed")
        .and_then(|c| c.as_bool())
        .unwrap_or(false);

    println!(
        "{} ({} books{})",
        name.bold().cyan(),
        books_count,
        if completed { ", completed" } else { "" }
    );
    println!("{} {}", "ID:".bold(), get_int(series, "id"));

    if let Some(desc) = series.get("description").and_then(|d| d.as_str()) {
        if !desc.is_empty() {
            println!("{}", strip_html(desc).dimmed());
        }
    }

    if let Some(books) = series.get("book_series").and_then(|bs| bs.as_array()) {
        println!();
        for bs in books {
            let pos = get_int(bs, "position");
            let book = &bs["book"];
            let title = get_str(book, "title");
            let book_id = get_int(book, "id");
            let authors = author_names(book);
            let year = get_int(book, "release_year");
            println!(
                "  {}. {} {} ({}) - {}",
                pos,
                format!("[{book_id}]").dimmed(),
                title.bold(),
                year,
                authors,
            );
        }
    }
}

pub fn print_user_profile(user: &Value) {
    let username = get_str(user, "username");
    let name = user.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let bio = user.get("bio").and_then(|b| b.as_str()).unwrap_or("");
    let location = user.get("location").and_then(|l| l.as_str()).unwrap_or("");
    let books = get_int(user, "books_count");
    let followers = get_int(user, "followers_count");
    let following = get_int(user, "followed_users_count");

    println!("{}", username.bold().cyan());
    if !name.is_empty() {
        println!("{} {}", "Name:".bold(), name);
    }
    println!("{} {}", "ID:".bold(), get_int(user, "id"));
    if !location.is_empty() {
        println!("{} {}", "Location:".bold(), location);
    }
    println!(
        "{} {} books | {} followers | {} following",
        "Stats:".bold(),
        books,
        followers,
        following
    );
    if !bio.is_empty() {
        println!("\n{}", bio.dimmed());
    }
}

pub fn print_reads(reads: &Value, book_id: i64) {
    let arr = match reads.as_array() {
        Some(a) => a,
        None => {
            println!("No read entries for book {}", book_id);
            return;
        }
    };

    if arr.is_empty() {
        println!("No read entries for book {}", book_id);
        return;
    }

    for read in arr {
        let id = get_int(read, "id");
        let started = get_str(read, "started_at");
        let finished = get_str(read, "finished_at");
        let progress = get_int(read, "progress_pages");

        println!(
            "  {} started: {} | finished: {} | progress: {} pages",
            format!("[{id}]").dimmed(),
            started,
            finished,
            progress,
        );
    }
}

pub fn print_journals(journals: &Value) {
    let arr = match journals.as_array() {
        Some(a) => a,
        None => {
            println!("No journal entries.");
            return;
        }
    };

    if arr.is_empty() {
        println!("No journal entries.");
        return;
    }

    for j in arr {
        let id = get_int(j, "id");
        let event = get_str(j, "event");
        let date = get_str(j, "action_at");
        let entry = j.get("entry").and_then(|e| e.as_str()).unwrap_or("");
        let book_title = j
            .get("book")
            .and_then(|b| b.get("title"))
            .and_then(|t| t.as_str())
            .unwrap_or("-");

        println!(
            "  {} [{}] {} - {}",
            format!("[{id}]").dimmed(),
            event,
            book_title.bold(),
            date,
        );
        if !entry.is_empty() {
            println!("    {}", entry.dimmed());
        }
    }
}

pub fn print_following(following: &Value) {
    let arr = match following.as_array() {
        Some(a) => a,
        None => {
            println!("Not following anyone.");
            return;
        }
    };

    if arr.is_empty() {
        println!("Not following anyone.");
        return;
    }

    for f in arr {
        let user = &f["user"];
        let id = get_int(user, "id");
        let username = get_str(user, "username");
        let name = user.get("name").and_then(|n| n.as_str()).unwrap_or("");
        println!(
            "  {} {} {}",
            format!("[{id}]").dimmed(),
            username.bold(),
            if name.is_empty() {
                String::new()
            } else {
                format!("({})", name)
            },
        );
    }
}

pub fn print_character(character: &Value) {
    let name = get_str(character, "name");
    let bio = character.get("biography").and_then(|b| b.as_str()).unwrap_or("");
    let books_count = get_int(character, "books_count");
    let lgtbq = character.get("is_lgbtq").and_then(|v| v.as_bool()).unwrap_or(false);
    let poc = character.get("is_poc").and_then(|v| v.as_bool()).unwrap_or(false);

    println!("{}", name.bold().cyan());
    println!("{} {}", "ID:".bold(), get_int(character, "id"));
    println!("{} {}", "Slug:".bold(), get_str(character, "slug"));
    
    let mut stats = format!("{books_count} books");
    if lgtbq { stats.push_str(" | LGBTQ+"); }
    if poc { stats.push_str(" | POC"); }
    println!("{} {}", "Info:".bold(), stats);

    if !bio.is_empty() {
        println!("\n{}", strip_html(bio).dimmed());
    }

    if let Some(books) = character.get("book_characters").and_then(|bc| bc.as_array()) {
        if !books.is_empty() {
            println!("\n{}", "Books featuring this character:".bold());
            for bc in books {
                let book = &bc["book"];
                let title = get_str(book, "title");
                let id = get_int(book, "id");
                let authors = author_names(book);
                println!("  {} {} - {}", format!("[{id}]").dimmed(), title.bold(), authors);
            }
        }
    }
}

pub fn print_tags(tags: &Value) {
    let arr = match tags.as_array() {
        Some(a) => a,
        None => {
            println!("No tags found.");
            return;
        }
    };

    if arr.is_empty() {
        println!("No tags found.");
        return;
    }

    println!("{:<6} {:<30} {:<10} {:<10}", "ID", "Tag", "Count", "Category");
    println!("{}", "-".repeat(60).dimmed());

    for tag in arr {
        let id = get_int(tag, "id");
        let name = get_str(tag, "tag");
        let count = get_int(tag, "count");
        let cat = get_int(tag, "tag_category_id");

        println!("{:<6} {:<30} {:<10} {:<10}", id.dimmed(), name.bold(), count, cat);
    }
}

pub fn print_notifications(notifications: &Value) {
    let arr = match notifications.as_array() {
        Some(a) => a,
        None => {
            println!("No notifications.");
            return;
        }
    };

    if arr.is_empty() {
        println!("No notifications.");
        return;
    }

    for n in arr {
        let title = get_str(n, "title");
        let desc = get_str(n, "description");
        let created = get_str(n, "created_at");
        let link = get_str(n, "link");

        println!("{} {}", "•".yellow(), title.bold());
        println!("  {}", desc.dimmed());
        if link != "-" {
            println!("  {}", link.blue().underline());
        }
        println!("  {}", created.dimmed());
        println!();
    }
}

pub fn print_editions(editions: &Value) {
    let arr = match editions.as_array() {
        Some(a) => a,
        None => {
            println!("No editions found.");
            return;
        }
    };

    if arr.is_empty() {
        println!("No editions found.");
        return;
    }

    for ed in arr {
        let title = get_str(ed, "title");
        let id = get_int(ed, "id");
        let format = get_str(ed, "edition_format");
        let pages = get_int(ed, "pages");
        let date = get_str(ed, "release_date");
        let publisher = get_str(&ed["publisher"], "name");
        let isbn10 = get_str(ed, "isbn_10");
        let isbn13 = get_str(ed, "isbn_13");

        println!("{} {}", format!("[{id}]").dimmed(), title.bold());
        println!("  {} | {} pages | {} | {}", format.yellow(), pages, date, publisher);
        if isbn10 != "-" || isbn13 != "-" {
            println!("  ISBN-10: {} | ISBN-13: {}", isbn10, isbn13);
        }
        println!();
    }
}

pub fn print_edition_detail(ed: &Value) {
    let title = get_str(ed, "title");
    let id = get_int(ed, "id");
    let format = get_str(ed, "edition_format");
    let pages = get_int(ed, "pages");
    let date = get_str(ed, "release_date");
    let publisher = get_str(&ed["publisher"], "name");
    let isbn10 = get_str(ed, "isbn_10");
    let isbn13 = get_str(ed, "isbn_13");

    println!("{} {}", "Edition:".bold().cyan(), title.bold());
    println!("{} {}", "ID:".bold(), id);
    println!("{} {}", "Format:".bold(), format.yellow());
    println!("{} {} pages", "Pages:".bold(), pages);
    println!("{} {}", "Released:".bold(), date);
    println!("{} {}", "Publisher:".bold(), publisher);
    println!("{} {}", "ISBN-10:".bold(), isbn10);
    println!("{} {}", "ISBN-13:".bold(), isbn13);

    if let Some(book) = ed.get("book") {
        println!("\n{}", "Parent Book:".bold());
        let b_title = get_str(book, "title");
        let b_id = get_int(book, "id");
        let authors = author_names(book);
        println!("  {} {} - {}", format!("[{b_id}]").dimmed(), b_title.bold(), authors);
    }
}

pub fn print_prompts(prompts: &Value) {
    let arr = match prompts.as_array() {
        Some(a) => a,
        None => {
            println!("No prompts found.");
            return;
        }
    };

    if arr.is_empty() {
        println!("No prompts found.");
        return;
    }

    for p in arr {
        let id = get_int(p, "id");
        let question = get_str(p, "question");
        let answers = get_int(p, "answers_count");

        println!("  {} {} ({} answers)", format!("[{id}]").dimmed(), question.bold(), answers);
    }
}

pub fn print_id_name_list(title: &str, items: &Value) {
    let arr = match items.as_array() {
        Some(a) => a,
        None => {
            println!("No items found for {}.", title);
            return;
        }
    };

    println!("{}:", title.bold());
    for item in arr {
        let id = get_int(item, "id");
        let name = get_str(item, "name");
        println!("  {} {}", format!("[{id}]").dimmed(), name);
    }
}
