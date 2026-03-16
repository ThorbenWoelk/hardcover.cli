use hc::client::HardcoverClient;
use mockito::{Matcher, Server};
use serde_json::json;

fn mock_graphql_response(data: serde_json::Value) -> String {
    json!({ "data": data }).to_string()
}

fn mock_graphql_error(message: &str) -> String {
    json!({
        "errors": [{
            "message": message,
            "extensions": {"code": "validation-failed"}
        }]
    })
    .to_string()
}

fn test_client(server: &Server) -> HardcoverClient {
    HardcoverClient::with_url("Bearer test-token".to_string(), server.url())
}

// --- me ---

#[tokio::test]
async fn me_returns_user() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/")
        .match_header("authorization", "Bearer test-token")
        .with_body(mock_graphql_response(json!({
            "me": [{"id": 83659, "username": "toto_hardcover"}]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let user = client.me().await.unwrap();

    assert_eq!(user["id"], 83659);
    assert_eq!(user["username"], "toto_hardcover");
    mock.assert_async().await;
}

// --- book_by_id ---

#[tokio::test]
async fn book_by_id_returns_book() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/")
        .match_body(Matcher::PartialJsonString(
            json!({"variables": {"id": 427578}}).to_string(),
        ))
        .with_body(mock_graphql_response(json!({
            "books_by_pk": {
                "id": 427578,
                "title": "Project Hail Mary",
                "subtitle": "A Novel",
                "slug": "project-hail-mary",
                "description": "A lone astronaut...",
                "release_year": 2021,
                "pages": 496,
                "rating": 4.5,
                "ratings_count": 5248,
                "users_count": 11239,
                "users_read_count": 7000,
                "cached_contributors": [{"author": {"name": "Andy Weir"}}],
                "cached_image": null,
                "cached_tags": [{"tag": "Science Fiction"}]
            }
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let book = client.book_by_id(427578).await.unwrap();

    assert_eq!(book["title"], "Project Hail Mary");
    assert_eq!(book["release_year"], 2021);
    assert_eq!(book["pages"], 496);
    mock.assert_async().await;
}

// --- book_by_slug ---

#[tokio::test]
async fn book_by_slug_returns_book() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/")
        .match_body(Matcher::PartialJsonString(
            json!({"variables": {"slug": "project-hail-mary"}}).to_string(),
        ))
        .with_body(mock_graphql_response(json!({
            "books": [{
                "id": 427578,
                "title": "Project Hail Mary",
                "slug": "project-hail-mary"
            }]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let book = client.book_by_slug("project-hail-mary").await.unwrap();

    assert_eq!(book["id"], 427578);
    mock.assert_async().await;
}

// --- search ---

#[tokio::test]
async fn search_books() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/")
        .match_body(Matcher::PartialJsonString(
            json!({"variables": {"q": "Project Hail Mary", "type": "Book", "per_page": 10, "page": 1}}).to_string(),
        ))
        .with_body(mock_graphql_response(json!({
            "search": {
                "results": {
                    "found": 7,
                    "hits": [
                        {"document": {"id": "427578", "title": "Project Hail Mary", "release_year": 2021, "author_names": ["Andy Weir"], "rating": 4.5, "users_count": 11239}}
                    ]
                }
            }
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let results = client
        .search("Project Hail Mary", "Book", 10, 1)
        .await
        .unwrap();

    assert_eq!(results["found"], 7);
    assert_eq!(results["hits"][0]["document"]["title"], "Project Hail Mary");
    mock.assert_async().await;
}

#[tokio::test]
async fn search_authors() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "search": {
                "results": {
                    "found": 12,
                    "hits": [
                        {"document": {"id": "204214", "name": "Brandon Sanderson", "books_count": 162}}
                    ]
                }
            }
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let results = client
        .search("Brandon Sanderson", "Author", 10, 1)
        .await
        .unwrap();

    assert_eq!(results["found"], 12);
    mock.assert_async().await;
}

// --- my_books ---

#[tokio::test]
async fn my_books_returns_list() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "me": [{
                "user_books": [
                    {
                        "id": 100,
                        "status_id": 3,
                        "rating": 3.0,
                        "review": null,
                        "owned": false,
                        "date_added": "2026-03-15",
                        "last_read_date": null,
                        "book": {
                            "id": 55654,
                            "title": "Dr. No",
                            "slug": "dr-no",
                            "release_year": 1958,
                            "pages": 256,
                            "rating": 3.5,
                            "cached_contributors": [{"author": {"name": "Ian Fleming"}}],
                            "cached_image": null
                        }
                    }
                ]
            }]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let books = client.my_books(None, 20, 0).await.unwrap();
    let arr = books.as_array().unwrap();

    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["book"]["title"], "Dr. No");
    assert_eq!(arr[0]["status_id"], 3);
    mock.assert_async().await;
}

#[tokio::test]
async fn my_books_with_status_filter() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/")
        .match_body(Matcher::PartialJsonString(
            json!({"variables": {"where": {"status_id": {"_eq": 2}}}}).to_string(),
        ))
        .with_body(mock_graphql_response(json!({
            "me": [{"user_books": [
                {
                    "id": 200,
                    "status_id": 2,
                    "rating": null,
                    "book": {"id": 1832368, "title": "AI Engineering"}
                }
            ]}]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let books = client.my_books(Some(2), 20, 0).await.unwrap();
    let arr = books.as_array().unwrap();

    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["status_id"], 2);
    mock.assert_async().await;
}

// --- my_books_all (pagination) ---

#[tokio::test]
async fn my_books_all_paginates() {
    let mut server = Server::new_async().await;

    // First page: 100 items (triggers next page fetch)
    let page1: Vec<serde_json::Value> = (0..100)
        .map(|i| {
            json!({
                "id": i,
                "status_id": 3,
                "book": {"id": i, "title": format!("Book {i}")}
            })
        })
        .collect();

    // Second page: 50 items (less than page size, stops pagination)
    let page2: Vec<serde_json::Value> = (100..150)
        .map(|i| {
            json!({
                "id": i,
                "status_id": 3,
                "book": {"id": i, "title": format!("Book {i}")}
            })
        })
        .collect();

    server
        .mock("POST", "/")
        .match_body(Matcher::PartialJsonString(
            json!({"variables": {"offset": 0}}).to_string(),
        ))
        .with_body(mock_graphql_response(json!({
            "me": [{"user_books": page1}]
        })))
        .create_async()
        .await;

    server
        .mock("POST", "/")
        .match_body(Matcher::PartialJsonString(
            json!({"variables": {"offset": 100}}).to_string(),
        ))
        .with_body(mock_graphql_response(json!({
            "me": [{"user_books": page2}]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let books = client.my_books_all(None).await.unwrap();
    let arr = books.as_array().unwrap();

    assert_eq!(arr.len(), 150);
}

// --- find_user_book_for_book ---

#[tokio::test]
async fn find_user_book_found() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "me": [{"user_books": [{
                "id": 42,
                "status_id": 3,
                "rating": 4.0,
                "book": {"id": 427578, "title": "Project Hail Mary"}
            }]}]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let result = client.find_user_book_for_book(427578).await.unwrap();

    assert!(result.is_some());
    assert_eq!(result.unwrap()["id"], 42);
}

#[tokio::test]
async fn find_user_book_not_found() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "me": [{"user_books": []}]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let result = client.find_user_book_for_book(999999).await.unwrap();

    assert!(result.is_none());
}

// --- lists ---

#[tokio::test]
async fn my_lists_returns_data() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "me": [{"lists": [
                {"id": 371666, "name": "Favorites", "description": null, "books_count": 16, "public": true, "ranked": true, "slug": "favorites"},
                {"id": 371379, "name": "Owned", "description": null, "books_count": 0, "public": false, "ranked": false, "slug": "owned"}
            ]}]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let lists = client.my_lists().await.unwrap();
    let arr = lists.as_array().unwrap();

    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["name"], "Favorites");
}

#[tokio::test]
async fn list_details_with_books() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "lists_by_pk": {
                "id": 371666,
                "name": "Favorites",
                "description": "My top picks",
                "books_count": 2,
                "public": true,
                "ranked": true,
                "list_books": [
                    {"id": 7516672, "position": 1, "book": {"id": 283379, "title": "All Quiet on the Western Front", "slug": "aqotwf", "release_year": 1929, "cached_contributors": [{"author": {"name": "Erich Maria Remarque"}}]}},
                    {"id": 7516654, "position": 2, "book": {"id": 435859, "title": "Steppenwolf", "slug": "steppenwolf", "release_year": 1927, "cached_contributors": [{"author": {"name": "Hermann Hesse"}}]}}
                ]
            }
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let list = client.list_details(371666).await.unwrap();

    assert_eq!(list["name"], "Favorites");
    assert_eq!(list["list_books"].as_array().unwrap().len(), 2);
}

// --- goals ---

#[tokio::test]
async fn my_goals_returns_data() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "me": [{"goals": [{
                "id": 77025,
                "goal": 12,
                "metric": "book",
                "description": "2026 Reading Goal",
                "start_date": "2025-12-31",
                "end_date": "2026-12-30"
            }]}]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let goals = client.my_goals().await.unwrap();
    let arr = goals.as_array().unwrap();

    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["goal"], 12);
}

// --- author ---

#[tokio::test]
async fn author_by_id_returns_data() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "authors_by_pk": {
                "id": 204214,
                "name": "Brandon Sanderson",
                "slug": "brandon-sanderson",
                "bio": "Born in 1975.",
                "born_year": 1975,
                "death_year": null,
                "location": "Utah, USA",
                "books_count": 162,
                "users_count": 12010,
                "cached_image": null
            }
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let author = client.author_by_id(204214).await.unwrap();

    assert_eq!(author["name"], "Brandon Sanderson");
    assert_eq!(author["books_count"], 162);
}

// --- series ---

#[tokio::test]
async fn series_by_id_returns_data() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "series_by_pk": {
                "id": 5497,
                "name": "The Cosmere",
                "slug": "the-cosmere",
                "description": "Shared cosmology.",
                "books_count": 38,
                "primary_books_count": 15,
                "is_completed": false,
                "book_series": [
                    {"position": 1.0, "book": {"id": 338931, "title": "Elantris", "slug": "elantris", "release_year": 2005, "cached_contributors": [{"author": {"name": "Brandon Sanderson"}}]}}
                ]
            }
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let series = client.series_by_id(5497).await.unwrap();

    assert_eq!(series["name"], "The Cosmere");
    assert_eq!(series["book_series"].as_array().unwrap().len(), 1);
}

// --- editions ---

#[tokio::test]
async fn editions_by_book_id_returns_data() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "editions": [
                {"id": 3274049, "title": "Project Hail Mary", "edition_format": "Hardcover", "pages": 496, "release_date": "2021-05-04", "isbn_10": "0593135202", "isbn_13": "9780593135204", "publisher": {"name": "Ballantine Books"}},
                {"id": 30530177, "title": "Project Hail Mary", "edition_format": "Paperback", "pages": 476, "release_date": "2022-10-04", "isbn_10": "0593135229", "isbn_13": "9780593135228", "publisher": {"name": "Ballantine Books"}}
            ]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let editions = client.editions_by_book_id(427578).await.unwrap();

    assert_eq!(editions.as_array().unwrap().len(), 2);
    assert_eq!(editions[0]["edition_format"], "Hardcover");
}

// --- edition_by_isbn ---

#[tokio::test]
async fn edition_by_isbn13_found() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "editions": [{
                "id": 3274049,
                "title": "Project Hail Mary",
                "edition_format": "Hardcover",
                "pages": 496,
                "release_date": "2021-05-04",
                "isbn_10": "0593135202",
                "isbn_13": "9780593135204",
                "publisher": {"name": "Ballantine Books"},
                "book": {"id": 427578, "title": "Project Hail Mary", "slug": "project-hail-mary", "cached_contributors": [{"author": {"name": "Andy Weir"}}]}
            }]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let result = client.edition_by_isbn("9780593135204").await.unwrap();

    assert!(result.is_some());
    assert_eq!(result.unwrap()["isbn_13"], "9780593135204");
}

#[tokio::test]
async fn edition_by_isbn10_found() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "editions": [{
                "id": 3274049,
                "title": "Project Hail Mary",
                "isbn_10": "0593135202"
            }]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let result = client.edition_by_isbn("0593135202").await.unwrap();

    assert!(result.is_some());
}

#[tokio::test]
async fn edition_by_isbn_not_found() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({"editions": []})))
        .create_async()
        .await;

    let client = test_client(&server);
    let result = client.edition_by_isbn("0000000000000").await.unwrap();

    assert!(result.is_none());
}

// --- user ---

#[tokio::test]
async fn user_by_username_returns_data() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "users": [{
                "id": 83659,
                "username": "toto_hardcover",
                "name": "Toto",
                "bio": null,
                "location": "Hamburg",
                "books_count": 322,
                "followers_count": 0,
                "followed_users_count": 0,
                "cached_image": null
            }]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let user = client.user_by_username("toto_hardcover").await.unwrap();

    assert_eq!(user["username"], "toto_hardcover");
    assert_eq!(user["location"], "Hamburg");
}

// --- character ---

#[tokio::test]
async fn character_by_id_returns_data() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "characters_by_pk": {
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
                    {"book": {"id": 328491, "title": "Harry Potter and the Philosopher's Stone", "slug": "hp-ps", "release_year": 1997, "cached_contributors": [{"author": {"name": "J.K. Rowling"}}]}}
                ]
            }
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let character = client.character_by_id(1).await.unwrap();

    assert_eq!(character["name"], "Harry Potter");
    assert_eq!(character["books_count"], 48);
}

#[tokio::test]
async fn character_by_slug_returns_data() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "characters": [{
                "id": 1,
                "name": "Harry Potter",
                "slug": "harry-potter",
                "books_count": 48,
                "book_characters": []
            }]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let character = client.character_by_slug("harry-potter").await.unwrap();

    assert_eq!(character["name"], "Harry Potter");
}

// --- tags ---

#[tokio::test]
async fn all_tags_returns_data() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "tags": [
                {"id": 5, "tag": "Fiction", "slug": "fiction", "count": 231149, "tag_category_id": 1},
                {"id": 2, "tag": "Fantasy", "slug": "fantasy", "count": 221169, "tag_category_id": 1}
            ]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let tags = client.all_tags(None, 50, 0).await.unwrap();

    assert_eq!(tags.as_array().unwrap().len(), 2);
    assert_eq!(tags[0]["tag"], "Fiction");
}

#[tokio::test]
async fn all_tags_with_category_filter() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .match_body(Matcher::PartialJsonString(
            json!({"variables": {"where": {"tag_category_id": {"_eq": 2}}}}).to_string(),
        ))
        .with_body(mock_graphql_response(json!({
            "tags": [
                {"id": 185, "tag": "Loveable Characters", "count": 53151, "tag_category_id": 2}
            ]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let tags = client.all_tags(Some(2), 5, 0).await.unwrap();

    assert_eq!(tags[0]["tag_category_id"], 2);
}

// --- notifications ---

#[tokio::test]
async fn my_notifications_returns_data() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "notifications": [
                {"id": 1, "title": "New follower", "description": "bookworm42 followed you", "link": "/users/bookworm42", "created_at": "2026-03-15T10:00:00Z", "notification_type_id": 1}
            ]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let notifications = client.my_notifications(20, 0).await.unwrap();

    assert_eq!(notifications.as_array().unwrap().len(), 1);
    assert_eq!(notifications[0]["title"], "New follower");
}

// --- prompts ---

#[tokio::test]
async fn all_prompts_returns_data() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "prompts": [
                {"id": 1, "question": "What are your favorite books of all time?", "description": null, "answers_count": 15451, "slug": "favorite-books"},
                {"id": 3, "question": "What were your favorite childhood books?", "description": null, "answers_count": 1140, "slug": "childhood-books"}
            ]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let prompts = client.all_prompts(20, 0).await.unwrap();

    assert_eq!(prompts.as_array().unwrap().len(), 2);
    assert_eq!(prompts[0]["answers_count"], 15451);
}

// --- platforms ---

#[tokio::test]
async fn all_platforms_returns_data() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "platforms": [
                {"id": 19, "name": "amazon"},
                {"id": 1, "name": "goodreads"},
                {"id": 27, "name": "storygraph"}
            ]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let platforms = client.all_platforms().await.unwrap();

    assert_eq!(platforms.as_array().unwrap().len(), 3);
}

// --- formats ---

#[tokio::test]
async fn all_formats_returns_data() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "reading_formats": [
                {"id": 3, "format": "Both"},
                {"id": 4, "format": "Ebook"},
                {"id": 2, "format": "Listened"},
                {"id": 1, "format": "Read"}
            ]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let formats = client.all_formats().await.unwrap();
    let arr = formats.as_array().unwrap();

    assert_eq!(arr.len(), 4);
    // Verify the format -> name remapping
    assert_eq!(arr[0]["name"], "Both");
    assert_eq!(arr[3]["name"], "Read");
}

// --- publishers ---

#[tokio::test]
async fn all_publishers_returns_data() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "publishers": [
                {"id": 1, "name": "Penguin"},
                {"id": 2, "name": "HarperCollins"}
            ]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let publishers = client.all_publishers(50, 0).await.unwrap();

    assert_eq!(publishers.as_array().unwrap().len(), 2);
}

// --- following ---

#[tokio::test]
async fn my_following_returns_data() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "me": [{"followed_users": [
                {"id": 1, "user": {"id": 100, "username": "bookworm42", "name": "Jane"}}
            ]}]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let following = client.my_following().await.unwrap();

    assert_eq!(following.as_array().unwrap().len(), 1);
    assert_eq!(following[0]["user"]["username"], "bookworm42");
}

#[tokio::test]
async fn my_following_empty() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "me": [{"followed_users": []}]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let following = client.my_following().await.unwrap();

    assert_eq!(following.as_array().unwrap().len(), 0);
}

// --- activity_feed ---

#[tokio::test]
async fn activity_feed_returns_data() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "activity_feed": [
                {
                    "id": 123,
                    "event": "status_update",
                    "created_at": "2026-03-15T20:00:00Z",
                    "user": {"username": "toto_hardcover"},
                    "book": {"title": "Project Hail Mary", "slug": "project-hail-mary"}
                }
            ]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let feed = client.activity_feed(20, 0).await.unwrap();

    assert_eq!(feed.as_array().unwrap().len(), 1);
    assert_eq!(feed[0]["event"], "status_update");
}

#[tokio::test]
async fn activity_feed_empty() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({"activity_feed": []})))
        .create_async()
        .await;

    let client = test_client(&server);
    let feed = client.activity_feed(20, 0).await.unwrap();

    assert_eq!(feed.as_array().unwrap().len(), 0);
}

// --- book_reads ---

#[tokio::test]
async fn book_reads_returns_data() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "me": [{"user_books": [{
                "id": 42,
                "user_book_reads": [
                    {"id": 4897015, "started_at": null, "finished_at": "2026-03-15", "edition_id": null, "progress": null, "progress_pages": null}
                ]
            }]}]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let reads = client.book_reads(55654).await.unwrap();
    let arr = reads.as_array().unwrap();

    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["finished_at"], "2026-03-15");
}

// --- journals ---

#[tokio::test]
async fn my_journals_returns_data() {
    let mut server = Server::new_async().await;

    // First call: me() for user ID
    server
        .mock("POST", "/")
        .match_body(Matcher::PartialJsonString(
            json!({"query": "{ me { id username } }"}).to_string(),
        ))
        .with_body(mock_graphql_response(json!({
            "me": [{"id": 83659, "username": "toto_hardcover"}]
        })))
        .create_async()
        .await;

    // Second call: reading_journals
    server
        .mock("POST", "/")
        .match_body(Matcher::PartialJsonString(
            json!({"variables": {"where": {"user_id": {"_eq": 83659}}}}).to_string(),
        ))
        .with_body(mock_graphql_response(json!({
            "reading_journals": [
                {
                    "id": 29626888,
                    "event": "rated",
                    "entry": null,
                    "action_at": "2026-03-15T20:44:58Z",
                    "book_id": 2487954,
                    "edition_id": null,
                    "book": {"id": 2487954, "title": "Die Scheibenwelt", "slug": "die-scheibenwelt"}
                }
            ]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let journals = client.my_journals(None, 20).await.unwrap();
    let arr = journals.as_array().unwrap();

    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["event"], "rated");
}

// --- trending ---

#[tokio::test]
async fn trending_books_returns_data() {
    let mut server = Server::new_async().await;

    // First call: get trending IDs
    server
        .mock("POST", "/")
        .match_body(Matcher::PartialJsonString(
            json!({"variables": {"limit": 10}}).to_string(),
        ))
        .with_body(mock_graphql_response(json!({
            "books_trending": {"ids": [427578, 446680]}
        })))
        .create_async()
        .await;

    // Second call: get book details for those IDs
    server
        .mock("POST", "/")
        .match_body(Matcher::PartialJsonString(
            json!({"variables": {"ids": [427578, 446680]}}).to_string(),
        ))
        .with_body(mock_graphql_response(json!({
            "books": [
                {"id": 427578, "title": "Project Hail Mary", "slug": "phm", "rating": 4.5, "users_count": 11239, "cached_contributors": [{"author": {"name": "Andy Weir"}}], "cached_image": null},
                {"id": 446680, "title": "Carl's Doomsday Scenario", "slug": "cds", "rating": 4.4, "users_count": 2663, "cached_contributors": [{"author": {"name": "Matt Dinniman"}}], "cached_image": null}
            ]
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let books = client.trending_books(10, 0, 7).await.unwrap();
    let arr = books.as_array().unwrap();

    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["title"], "Project Hail Mary");
}

#[tokio::test]
async fn trending_books_empty() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_response(json!({
            "books_trending": {"ids": []}
        })))
        .create_async()
        .await;

    let client = test_client(&server);
    let books = client.trending_books(10, 0, 7).await.unwrap();

    assert_eq!(books.as_array().unwrap().len(), 0);
}

// --- Error handling ---

#[tokio::test]
async fn graphql_error_returns_err() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body(mock_graphql_error("field 'foo' not found"))
        .create_async()
        .await;

    let client = test_client(&server);
    let result = client.me().await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("GraphQL errors"));
}

#[tokio::test]
async fn http_error_returns_err() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_status(500)
        .with_body("Internal Server Error")
        .create_async()
        .await;

    let client = test_client(&server);
    let result = client.me().await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API returned"));
}

#[tokio::test]
async fn invalid_json_returns_err() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/")
        .with_body("not json at all")
        .create_async()
        .await;

    let client = test_client(&server);
    let result = client.me().await;

    assert!(result.is_err());
}
