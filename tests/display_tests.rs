use hc::display::*;
use serde_json::json;

// --- parse_status ---

#[test]
fn parse_status_full_names() {
    assert_eq!(parse_status("want-to-read"), Some(1));
    assert_eq!(parse_status("reading"), Some(2));
    assert_eq!(parse_status("currently-reading"), Some(2));
    assert_eq!(parse_status("read"), Some(3));
    assert_eq!(parse_status("paused"), Some(4));
    assert_eq!(parse_status("dnf"), Some(5));
    assert_eq!(parse_status("did-not-finish"), Some(5));
    assert_eq!(parse_status("ignored"), Some(6));
}

#[test]
fn parse_status_short_aliases() {
    assert_eq!(parse_status("wtr"), Some(1));
    assert_eq!(parse_status("cr"), Some(2));
    assert_eq!(parse_status("r"), Some(3));
    assert_eq!(parse_status("p"), Some(4));
    assert_eq!(parse_status("i"), Some(6));
}

#[test]
fn parse_status_numeric_strings() {
    assert_eq!(parse_status("1"), Some(1));
    assert_eq!(parse_status("2"), Some(2));
    assert_eq!(parse_status("3"), Some(3));
    assert_eq!(parse_status("4"), Some(4));
    assert_eq!(parse_status("5"), Some(5));
    assert_eq!(parse_status("6"), Some(6));
}

#[test]
fn parse_status_case_insensitive() {
    assert_eq!(parse_status("Want-To-Read"), Some(1));
    assert_eq!(parse_status("READING"), Some(2));
    assert_eq!(parse_status("DNF"), Some(5));
    assert_eq!(parse_status("WTR"), Some(1));
}

#[test]
fn parse_status_invalid() {
    assert_eq!(parse_status("invalid"), None);
    assert_eq!(parse_status(""), None);
    assert_eq!(parse_status("7"), None);
    assert_eq!(parse_status("0"), None);
}

// --- status_name ---

#[test]
fn status_name_all_ids() {
    assert_eq!(status_name(1), "Want to Read");
    assert_eq!(status_name(2), "Currently Reading");
    assert_eq!(status_name(3), "Read");
    assert_eq!(status_name(4), "Paused");
    assert_eq!(status_name(5), "Did Not Finish");
    assert_eq!(status_name(6), "Ignored");
    assert_eq!(status_name(0), "Unknown");
    assert_eq!(status_name(99), "Unknown");
}

// --- print_user ---

#[test]
fn print_user_with_real_data() {
    let user = json!({
        "id": 83659,
        "username": "toto_hardcover"
    });
    // Should not panic
    print_user(&user);
}

#[test]
fn print_user_with_missing_fields() {
    let user = json!({});
    print_user(&user);
}

// --- print_book_detail ---

#[test]
fn print_book_detail_full() {
    let book = json!({
        "id": 427578,
        "title": "Project Hail Mary",
        "subtitle": "A Novel",
        "slug": "project-hail-mary",
        "description": "A lone astronaut must save the earth.",
        "release_year": 2021,
        "pages": 496,
        "rating": 4.5,
        "ratings_count": 5248,
        "users_count": 11239,
        "cached_contributors": [
            {"author": {"name": "Andy Weir"}}
        ],
        "cached_tags": [
            {"tag": "Science Fiction"},
            {"tag": "Adventure"}
        ]
    });
    print_book_detail(&book);
}

#[test]
fn print_book_detail_minimal() {
    let book = json!({
        "id": 1,
        "title": "Untitled"
    });
    print_book_detail(&book);
}

#[test]
fn print_book_detail_html_description() {
    let book = json!({
        "id": 1,
        "title": "Test",
        "description": "<p>Hello <b>world</b></p><br>New line"
    });
    print_book_detail(&book);
}

// --- print_search_results ---

#[test]
fn print_search_results_books() {
    let results = json!({
        "found": 7,
        "hits": [
            {
                "document": {
                    "id": "427578",
                    "title": "Project Hail Mary",
                    "release_year": 2021,
                    "author_names": ["Andy Weir"],
                    "rating": 4.5,
                    "users_count": 11239
                }
            },
            {
                "document": {
                    "id": "2485594",
                    "title": "Project Hail Mary",
                    "release_year": null,
                    "author_names": [],
                    "rating": 0.0,
                    "users_count": 1
                }
            }
        ]
    });
    print_search_results(&results, "Book");
}

#[test]
fn print_search_results_authors() {
    let results = json!({
        "found": 2,
        "hits": [
            {
                "document": {
                    "id": "204214",
                    "name": "Brandon Sanderson",
                    "books_count": 162
                }
            }
        ]
    });
    print_search_results(&results, "Author");
}

#[test]
fn print_search_results_series() {
    let results = json!({
        "found": 1,
        "hits": [
            {
                "document": {
                    "id": "5497",
                    "name": "The Cosmere",
                    "books_count": 39
                }
            }
        ]
    });
    print_search_results(&results, "Series");
}

#[test]
fn print_search_results_users() {
    let results = json!({
        "found": 2,
        "hits": [
            {
                "document": {
                    "id": "83659",
                    "username": "toto_hardcover",
                    "books_count": 322
                }
            }
        ]
    });
    print_search_results(&results, "User");
}

#[test]
fn print_search_results_lists() {
    let results = json!({
        "found": 2600,
        "hits": [
            {
                "document": {
                    "id": "71902",
                    "name": "Favorites",
                    "books_count": 3
                }
            }
        ]
    });
    print_search_results(&results, "List");
}

#[test]
fn print_search_results_empty() {
    let results = json!({
        "found": 0,
        "hits": []
    });
    print_search_results(&results, "Book");
}

#[test]
fn print_search_results_no_hits_key() {
    let results = json!({});
    print_search_results(&results, "Book");
}

// --- print_my_books ---

#[test]
fn print_my_books_with_data() {
    let books = json!([
        {
            "id": 100,
            "status_id": 3,
            "rating": 3.0,
            "owned": false,
            "book": {
                "id": 55654,
                "title": "Dr. No",
                "slug": "dr-no",
                "release_year": 1958,
                "pages": 256,
                "rating": 3.5,
                "cached_contributors": [{"author": {"name": "Ian Fleming"}}]
            }
        },
        {
            "id": 101,
            "status_id": 2,
            "rating": null,
            "owned": true,
            "book": {
                "id": 1832368,
                "title": "AI Engineering",
                "slug": "ai-engineering",
                "release_year": 2024,
                "pages": 400,
                "rating": 4.2,
                "cached_contributors": [{"author": {"name": "Chip Huyen"}}]
            }
        }
    ]);
    print_my_books(&books);
}

#[test]
fn print_my_books_empty_array() {
    let books = json!([]);
    print_my_books(&books);
}

#[test]
fn print_my_books_null() {
    let books = json!(null);
    print_my_books(&books);
}

// --- print_lists ---

#[test]
fn print_lists_with_data() {
    let lists = json!([
        {
            "id": 371666,
            "name": "Favorites",
            "description": "My top picks",
            "books_count": 16,
            "public": true,
            "ranked": true,
            "slug": "favorites"
        },
        {
            "id": 371379,
            "name": "Owned",
            "description": null,
            "books_count": 0,
            "public": false,
            "ranked": false,
            "slug": "owned"
        }
    ]);
    print_lists(&lists);
}

#[test]
fn print_lists_empty() {
    print_lists(&json!([]));
}

#[test]
fn print_lists_null() {
    print_lists(&json!(null));
}

// --- print_list_detail ---

#[test]
fn print_list_detail_with_books() {
    let list = json!({
        "id": 371666,
        "name": "Favorites",
        "description": "My favorite books of all time",
        "books_count": 2,
        "public": true,
        "ranked": true,
        "list_books": [
            {
                "id": 7516672,
                "position": 1,
                "book": {
                    "id": 283379,
                    "title": "All Quiet on the Western Front",
                    "slug": "all-quiet-on-the-western-front",
                    "release_year": 1929,
                    "cached_contributors": [{"author": {"name": "Erich Maria Remarque"}}]
                }
            },
            {
                "id": 7516654,
                "position": 2,
                "book": {
                    "id": 435859,
                    "title": "Steppenwolf",
                    "slug": "steppenwolf",
                    "release_year": 1927,
                    "cached_contributors": [{"author": {"name": "Hermann Hesse"}}]
                }
            }
        ]
    });
    print_list_detail(&list);
}

#[test]
fn print_list_detail_empty_list() {
    let list = json!({
        "id": 1,
        "name": "Empty List",
        "books_count": 0,
        "list_books": []
    });
    print_list_detail(&list);
}

// --- print_trending ---

#[test]
fn print_trending_with_data() {
    let books = json!([
        {
            "id": 427578,
            "title": "Project Hail Mary",
            "slug": "project-hail-mary",
            "rating": 4.5,
            "users_count": 11239,
            "cached_contributors": [{"author": {"name": "Andy Weir"}}]
        },
        {
            "id": 446680,
            "title": "Carl's Doomsday Scenario",
            "slug": "carls-doomsday-scenario",
            "rating": 4.4,
            "users_count": 2663,
            "cached_contributors": [{"author": {"name": "Matt Dinniman"}}]
        }
    ]);
    print_trending(&books);
}

#[test]
fn print_trending_empty() {
    print_trending(&json!([]));
}

#[test]
fn print_trending_null() {
    print_trending(&json!(null));
}

// --- print_goals ---

#[test]
fn print_goals_with_data() {
    let goals = json!([
        {
            "id": 77025,
            "goal": 12,
            "metric": "book",
            "description": "2026 Reading Goal",
            "start_date": "2025-12-31",
            "end_date": "2026-12-30"
        }
    ]);
    print_goals(&goals);
}

#[test]
fn print_goals_empty() {
    print_goals(&json!([]));
}

#[test]
fn print_goals_null() {
    print_goals(&json!(null));
}

// --- print_feed ---

#[test]
fn print_feed_with_data() {
    let feed = json!([
        {
            "id": 123,
            "event": "status_update",
            "created_at": "2026-03-15T20:00:00Z",
            "user": {"username": "toto_hardcover"},
            "book": {"title": "Project Hail Mary", "slug": "project-hail-mary"}
        }
    ]);
    print_feed(&feed);
}

#[test]
fn print_feed_empty() {
    print_feed(&json!([]));
}

#[test]
fn print_feed_null() {
    print_feed(&json!(null));
}

// --- print_author ---

#[test]
fn print_author_full() {
    let author = json!({
        "id": 204214,
        "name": "Brandon Sanderson",
        "slug": "brandon-sanderson",
        "bio": "Brandon Sanderson was born in December 1975 in Lincoln, Nebraska.",
        "born_year": 1975,
        "death_year": null,
        "location": "Utah, USA",
        "books_count": 162,
        "users_count": 12010
    });
    print_author(&author);
}

#[test]
fn print_author_deceased() {
    let author = json!({
        "id": 1,
        "name": "Terry Pratchett",
        "slug": "terry-pratchett",
        "bio": null,
        "born_year": 1948,
        "death_year": 2015,
        "location": null,
        "books_count": 80,
        "users_count": 5000
    });
    print_author(&author);
}

// --- print_series ---

#[test]
fn print_series_with_books() {
    let series = json!({
        "id": 5497,
        "name": "The Cosmere",
        "slug": "the-cosmere",
        "description": "All of my Cosmere books share a single creation myth.",
        "books_count": 38,
        "primary_books_count": 15,
        "is_completed": false,
        "book_series": [
            {
                "position": 1.0,
                "book": {
                    "id": 338931,
                    "title": "Elantris",
                    "slug": "elantris",
                    "release_year": 2005,
                    "cached_contributors": [{"author": {"name": "Brandon Sanderson"}}]
                }
            },
            {
                "position": 2.0,
                "book": {
                    "id": 369692,
                    "title": "Mistborn: The Final Empire",
                    "slug": "mistborn-the-final-empire",
                    "release_year": 2006,
                    "cached_contributors": [{"author": {"name": "Brandon Sanderson"}}]
                }
            }
        ]
    });
    print_series(&series);
}

// --- print_user_profile ---

#[test]
fn print_user_profile_full() {
    let user = json!({
        "id": 83659,
        "username": "toto_hardcover",
        "name": "Toto",
        "bio": null,
        "location": "Hamburg",
        "books_count": 322,
        "followers_count": 0,
        "followed_users_count": 0
    });
    print_user_profile(&user);
}

#[test]
fn print_user_profile_minimal() {
    let user = json!({
        "id": 1,
        "username": "testuser"
    });
    print_user_profile(&user);
}

// --- print_reads ---

#[test]
fn print_reads_with_entries() {
    let reads = json!([
        {
            "id": 4897015,
            "started_at": null,
            "finished_at": "2026-03-15",
            "edition_id": null,
            "progress": null,
            "progress_pages": null
        }
    ]);
    print_reads(&reads, 55654);
}

#[test]
fn print_reads_multiple() {
    let reads = json!([
        {
            "id": 100,
            "started_at": "2026-01-01",
            "finished_at": "2026-01-15",
            "progress_pages": 496
        },
        {
            "id": 101,
            "started_at": "2026-03-01",
            "finished_at": null,
            "progress_pages": 120
        }
    ]);
    print_reads(&reads, 427578);
}

#[test]
fn print_reads_empty() {
    print_reads(&json!([]), 1);
}

#[test]
fn print_reads_null() {
    print_reads(&json!(null), 1);
}

// --- print_journals ---

#[test]
fn print_journals_with_entries() {
    let journals = json!([
        {
            "id": 29626888,
            "event": "rated",
            "entry": null,
            "action_at": "2026-03-15T20:44:58.183731+00:00",
            "book_id": 2487954,
            "edition_id": null,
            "book": {
                "id": 2487954,
                "title": "Die Scheibenwelt",
                "slug": "die-scheibenwelt"
            }
        },
        {
            "id": 29626884,
            "event": "rated",
            "entry": "Great book!",
            "action_at": "2026-03-15T20:44:56.327653+00:00",
            "book_id": 382859,
            "edition_id": null,
            "book": {
                "id": 382859,
                "title": "Fall of Giants",
                "slug": "fall-of-giants"
            }
        }
    ]);
    print_journals(&journals);
}

#[test]
fn print_journals_empty() {
    print_journals(&json!([]));
}

#[test]
fn print_journals_null() {
    print_journals(&json!(null));
}

// --- print_following ---

#[test]
fn print_following_with_data() {
    let following = json!([
        {
            "id": 1,
            "user": {
                "id": 100,
                "username": "bookworm42",
                "name": "Jane Doe"
            }
        },
        {
            "id": 2,
            "user": {
                "id": 200,
                "username": "reader99",
                "name": null
            }
        }
    ]);
    print_following(&following);
}

#[test]
fn print_following_empty() {
    print_following(&json!([]));
}

#[test]
fn print_following_null() {
    print_following(&json!(null));
}

// --- print_character ---

#[test]
fn print_character_full() {
    let character = json!({
        "id": 1,
        "name": "Harry Potter",
        "slug": "harry-potter",
        "biography": null,
        "gender_id": null,
        "is_lgbtq": false,
        "is_poc": false,
        "books_count": 48,
        "cached_tags": [],
        "book_characters": [
            {
                "book": {
                    "id": 328491,
                    "title": "Harry Potter and the Philosopher's Stone",
                    "slug": "harry-potter-philosophers-stone",
                    "release_year": 1997,
                    "cached_contributors": [{"author": {"name": "J.K. Rowling"}}]
                }
            }
        ]
    });
    print_character(&character);
}

#[test]
fn print_character_lgbtq_poc() {
    let character = json!({
        "id": 99,
        "name": "Test Character",
        "slug": "test-character",
        "biography": "A <b>test</b> character.",
        "is_lgbtq": true,
        "is_poc": true,
        "books_count": 3,
        "book_characters": []
    });
    print_character(&character);
}

// --- print_tags ---

#[test]
fn print_tags_with_data() {
    let tags = json!([
        {"id": 5, "tag": "Fiction", "slug": "fiction", "count": 231149, "tag_category_id": 1},
        {"id": 2, "tag": "Fantasy", "slug": "fantasy", "count": 221169, "tag_category_id": 1},
        {"id": 6, "tag": "Science Fiction", "slug": "science-fiction", "count": 101207, "tag_category_id": 1}
    ]);
    print_tags(&tags);
}

#[test]
fn print_tags_empty() {
    print_tags(&json!([]));
}

#[test]
fn print_tags_null() {
    print_tags(&json!(null));
}

// --- print_notifications ---

#[test]
fn print_notifications_with_data() {
    let notifications = json!([
        {
            "id": 1,
            "title": "New follower",
            "description": "bookworm42 followed you",
            "link": "/users/bookworm42",
            "created_at": "2026-03-15T10:00:00Z",
            "notification_type_id": 1
        }
    ]);
    print_notifications(&notifications);
}

#[test]
fn print_notifications_no_link() {
    let notifications = json!([
        {
            "id": 2,
            "title": "Goal reached",
            "description": "You completed your 2026 reading goal!",
            "link": null,
            "created_at": "2026-12-30T00:00:00Z",
            "notification_type_id": 2
        }
    ]);
    print_notifications(&notifications);
}

#[test]
fn print_notifications_empty() {
    print_notifications(&json!([]));
}

#[test]
fn print_notifications_null() {
    print_notifications(&json!(null));
}

// --- print_editions ---

#[test]
fn print_editions_with_data() {
    let editions = json!([
        {
            "id": 3274049,
            "title": "Project Hail Mary",
            "edition_format": "Hardcover",
            "pages": 496,
            "release_date": "2021-05-04",
            "isbn_10": "0593135202",
            "isbn_13": "9780593135204",
            "publisher": {"name": "Ballantine Books"}
        },
        {
            "id": 30530177,
            "title": "Project Hail Mary",
            "edition_format": "Paperback",
            "pages": 476,
            "release_date": "2022-10-04",
            "isbn_10": "0593135229",
            "isbn_13": "9780593135228",
            "publisher": {"name": "Ballantine Books"}
        }
    ]);
    print_editions(&editions);
}

#[test]
fn print_editions_no_isbn() {
    let editions = json!([
        {
            "id": 1,
            "title": "Some Book",
            "edition_format": "ebook",
            "pages": null,
            "release_date": null,
            "isbn_10": null,
            "isbn_13": null,
            "publisher": {"name": null}
        }
    ]);
    print_editions(&editions);
}

#[test]
fn print_editions_empty() {
    print_editions(&json!([]));
}

// --- print_edition_detail ---

#[test]
fn print_edition_detail_full() {
    let ed = json!({
        "id": 3274049,
        "title": "Project Hail Mary",
        "edition_format": "Hardcover",
        "pages": 496,
        "release_date": "2021-05-04",
        "isbn_10": "0593135202",
        "isbn_13": "9780593135204",
        "publisher": {"name": "Ballantine Books"},
        "book": {
            "id": 427578,
            "title": "Project Hail Mary",
            "slug": "project-hail-mary",
            "cached_contributors": [{"author": {"name": "Andy Weir"}}]
        }
    });
    print_edition_detail(&ed);
}

#[test]
fn print_edition_detail_no_parent_book() {
    let ed = json!({
        "id": 1,
        "title": "Orphan Edition",
        "edition_format": "Paperback",
        "pages": 200,
        "release_date": "2020-01-01",
        "isbn_10": null,
        "isbn_13": null,
        "publisher": {"name": "Publisher"}
    });
    print_edition_detail(&ed);
}

// --- print_prompts ---

#[test]
fn print_prompts_with_data() {
    let prompts = json!([
        {
            "id": 1,
            "question": "What are your favorite books of all time?",
            "description": null,
            "answers_count": 15451,
            "slug": "favorite-books"
        },
        {
            "id": 3,
            "question": "What were your favorite childhood books?",
            "description": "Share your nostalgia",
            "answers_count": 1140,
            "slug": "childhood-books"
        }
    ]);
    print_prompts(&prompts);
}

#[test]
fn print_prompts_empty() {
    print_prompts(&json!([]));
}

#[test]
fn print_prompts_null() {
    print_prompts(&json!(null));
}

// --- print_id_name_list ---

#[test]
fn print_id_name_list_formats() {
    let items = json!([
        {"id": 1, "name": "Read"},
        {"id": 2, "name": "Listened"},
        {"id": 3, "name": "Both"},
        {"id": 4, "name": "Ebook"}
    ]);
    print_id_name_list("Reading Formats", &items);
}

#[test]
fn print_id_name_list_platforms() {
    let items = json!([
        {"id": 19, "name": "amazon"},
        {"id": 1, "name": "goodreads"},
        {"id": 3, "name": "openlibrary"},
        {"id": 27, "name": "storygraph"}
    ]);
    print_id_name_list("Platforms", &items);
}

#[test]
fn print_id_name_list_null() {
    print_id_name_list("Test", &json!(null));
}
