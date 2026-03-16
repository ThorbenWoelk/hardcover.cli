use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::{Value, json};

const API_URL: &str = "https://api.hardcover.app/v1/graphql";

pub struct HardcoverClient {
    client: Client,
    token: String,
}

impl HardcoverClient {
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(),
            token,
        }
    }

    pub async fn query(&self, query: &str, variables: Option<Value>) -> Result<Value> {
        let body = if let Some(vars) = variables {
            json!({ "query": query, "variables": vars })
        } else {
            json!({ "query": query })
        };

        let resp = self
            .client
            .post(API_URL)
            .header("Content-Type", "application/json")
            .header("Authorization", &self.token)
            .json(&body)
            .send()
            .await
            .context("Failed to send request to Hardcover API")?;

        let status = resp.status();
        let text = resp.text().await.context("Failed to read response body")?;

        if !status.is_success() {
            anyhow::bail!("API returned {status}: {text}");
        }

        let data: Value = serde_json::from_str(&text).context("Failed to parse JSON response")?;

        if let Some(errors) = data.get("errors") {
            anyhow::bail!("GraphQL errors: {}", serde_json::to_string_pretty(errors)?);
        }

        Ok(data["data"].clone())
    }

    pub async fn me(&self) -> Result<Value> {
        let query = r#"{ me { id username } }"#;
        let data = self.query(query, None).await?;
        Ok(data["me"][0].clone())
    }

    pub async fn book_by_id(&self, id: i64) -> Result<Value> {
        let query = r#"query ($id: Int!) {
            books_by_pk(id: $id) {
                id title subtitle slug description
                release_year pages rating ratings_count
                users_count users_read_count
                cached_contributors cached_image
                cached_tags
            }
        }"#;
        let vars = json!({ "id": id });
        let data = self.query(query, Some(vars)).await?;
        Ok(data["books_by_pk"].clone())
    }

    pub async fn book_by_slug(&self, slug: &str) -> Result<Value> {
        let query = r#"query ($slug: String!) {
            books(where: { slug: { _eq: $slug } }, limit: 1) {
                id title subtitle slug description
                release_year pages rating ratings_count
                users_count users_read_count
                cached_contributors cached_image
                cached_tags
            }
        }"#;
        let vars = json!({ "slug": slug });
        let data = self.query(query, Some(vars)).await?;
        Ok(data["books"][0].clone())
    }

    pub async fn my_books(&self, status_id: Option<i32>, limit: i32, offset: i32) -> Result<Value> {
        let query = r#"query ($limit: Int!, $offset: Int!, $where: user_books_bool_exp) {
            me {
                user_books(limit: $limit, offset: $offset, order_by: {updated_at: desc}, where: $where) {
                    id status_id rating review owned
                    date_added last_read_date
                    book {
                        id title slug release_year pages rating
                        cached_contributors cached_image
                    }
                }
            }
        }"#;

        let mut where_obj = json!({});
        if let Some(sid) = status_id {
            where_obj = json!({ "status_id": { "_eq": sid } });
        }

        let vars = json!({
            "limit": limit,
            "offset": offset,
            "where": where_obj
        });

        let data = self.query(query, Some(vars)).await?;
        Ok(data["me"][0]["user_books"].clone())
    }

    pub async fn my_books_all(&self, status_id: Option<i32>) -> Result<Value> {
        let page_size = 100;
        let mut all = Vec::new();
        let mut offset = 0;

        loop {
            let page = self.my_books(status_id, page_size, offset).await?;
            let arr = page.as_array().cloned().unwrap_or_default();
            let count = arr.len();
            all.extend(arr);
            if count < page_size as usize {
                break;
            }
            offset += page_size;
        }

        Ok(Value::Array(all))
    }

    pub async fn create_book(
        &self,
        title: &str,
        pages: Option<i32>,
        release_date: Option<&str>,
        description: Option<&str>,
    ) -> Result<Value> {
        let query = r#"mutation ($input: createBookInput!) {
            createBook(input: $input) {
                book { id title slug }
            }
        }"#;
        let mut input = json!({ "title": title });
        if let Some(p) = pages {
            input["pages"] = json!(p);
        }
        if let Some(d) = release_date {
            input["release_date"] = json!(d);
        }
        if let Some(desc) = description {
            input["description"] = json!(desc);
        }
        let vars = json!({ "input": input });
        let data = self.query(query, Some(vars)).await?;
        Ok(data["createBook"]["book"].clone())
    }

    pub async fn insert_user_book(&self, object: Value) -> Result<Value> {
        let query = r#"mutation ($object: UserBookCreateInput!) {
            insert_user_book(object: $object) { id status_id book { id title } }
        }"#;
        let vars = json!({ "object": object });
        self.query(query, Some(vars)).await
    }

    pub async fn update_user_book(&self, user_book_id: i64, updates: Value) -> Result<Value> {
        let query = r#"mutation ($id: Int!, $object: UserBookUpdateInput!) {
            update_user_book(id: $id, object: $object) { id status_id rating }
        }"#;
        let vars = json!({ "id": user_book_id, "object": updates });
        self.query(query, Some(vars)).await
    }

    pub async fn delete_user_book(&self, user_book_id: i64) -> Result<Value> {
        let query = r#"mutation ($id: Int!) {
            delete_user_book(id: $id) { id }
        }"#;
        let vars = json!({ "id": user_book_id });
        self.query(query, Some(vars)).await
    }

    pub async fn find_user_book_for_book(&self, book_id: i64) -> Result<Option<Value>> {
        let query = r#"query ($book_id: Int!) {
            me { user_books(where: { book_id: { _eq: $book_id } }, limit: 1) {
                id status_id rating book { id title }
            } }
        }"#;
        let vars = json!({ "book_id": book_id });
        let data = self.query(query, Some(vars)).await?;
        let books = &data["me"][0]["user_books"];
        if let Some(arr) = books.as_array() {
            Ok(arr.first().cloned())
        } else {
            Ok(None)
        }
    }

    pub async fn my_lists(&self) -> Result<Value> {
        let query = r#"{ me { lists(order_by: { updated_at: desc }) {
            id name description books_count public ranked slug
        } } }"#;
        let data = self.query(query, None).await?;
        Ok(data["me"][0]["lists"].clone())
    }

    pub async fn list_details(&self, list_id: i64) -> Result<Value> {
        let query = r#"query ($id: Int!) {
            lists_by_pk(id: $id) {
                id name description books_count public ranked
                list_books(order_by: { position: asc }) {
                    id position book { id title slug release_year cached_contributors }
                }
            }
        }"#;
        let vars = json!({ "id": list_id });
        let data = self.query(query, Some(vars)).await?;
        Ok(data["lists_by_pk"].clone())
    }

    pub async fn create_list(
        &self,
        name: &str,
        description: Option<&str>,
        ranked: Option<bool>,
        privacy: Option<i32>,
    ) -> Result<Value> {
        let query = r#"mutation ($object: ListInput!) {
            insert_list(object: $object) { id name slug }
        }"#;
        let mut obj = json!({ "name": name });
        if let Some(desc) = description {
            obj["description"] = json!(desc);
        }
        if let Some(r) = ranked {
            obj["ranked"] = json!(r);
        }
        if let Some(p) = privacy {
            obj["privacy_setting_id"] = json!(p);
        }
        let vars = json!({ "object": obj });
        self.query(query, Some(vars)).await
    }

    pub async fn update_list(
        &self,
        list_id: i64,
        name: Option<&str>,
        description: Option<&str>,
        ranked: Option<bool>,
        privacy: Option<i32>,
    ) -> Result<Value> {
        let query = r#"mutation ($id: Int!, $object: ListInput!) {
            update_list(id: $id, object: $object) { id name }
        }"#;
        let mut obj = json!({});
        if let Some(n) = name {
            obj["name"] = json!(n);
        }
        if let Some(d) = description {
            obj["description"] = json!(d);
        }
        if let Some(r) = ranked {
            obj["ranked"] = json!(r);
        }
        if let Some(p) = privacy {
            obj["privacy_setting_id"] = json!(p);
        }
        let vars = json!({ "id": list_id, "object": obj });
        self.query(query, Some(vars)).await
    }

    pub async fn delete_list(&self, list_id: i64) -> Result<Value> {
        let query = r#"mutation ($id: Int!) {
            delete_list(id: $id) { id }
        }"#;
        let vars = json!({ "id": list_id });
        self.query(query, Some(vars)).await
    }

    pub async fn add_book_to_list(
        &self,
        list_id: i64,
        book_id: i64,
        position: Option<i32>,
        edition_id: Option<i64>,
    ) -> Result<Value> {
        let query = r#"mutation ($object: ListBookInput!) {
            insert_list_book(object: $object) { id list_id book { id title } }
        }"#;
        let mut obj = json!({ "list_id": list_id, "book_id": book_id });
        if let Some(p) = position {
            obj["position"] = json!(p);
        }
        if let Some(e) = edition_id {
            obj["edition_id"] = json!(e);
        }
        let vars = json!({ "object": obj });
        self.query(query, Some(vars)).await
    }

    pub async fn remove_list_book(&self, list_book_id: i64) -> Result<Value> {
        let query = r#"mutation ($id: Int!) {
            delete_list_book(id: $id) { id }
        }"#;
        let vars = json!({ "id": list_book_id });
        self.query(query, Some(vars)).await
    }

    pub async fn my_goals(&self) -> Result<Value> {
        let query = r#"{ me { goals(order_by: { start_date: desc }) {
            id goal metric description start_date end_date
        } } }"#;
        let data = self.query(query, None).await?;
        Ok(data["me"][0]["goals"].clone())
    }

    pub async fn activity_feed(&self, limit: i32, offset: i32) -> Result<Value> {
        let query = r#"query ($limit: Int!, $offset: Int!) {
            activity_feed(args: { feed_limit: $limit, feed_offset: $offset }, order_by: { created_at: desc }) {
                id event created_at
                user { username }
                book { title slug }
            }
        }"#;
        let vars = json!({ "limit": limit, "offset": offset });
        let data = self.query(query, Some(vars)).await?;
        Ok(data["activity_feed"].clone())
    }

    // --- Search (multi-type) ---

    pub async fn search(
        &self,
        search_query: &str,
        query_type: &str,
        per_page: u32,
        page: u32,
    ) -> Result<Value> {
        let query = r#"query ($q: String!, $type: String!, $per_page: Int!, $page: Int!) {
            search(query: $q, query_type: $type, per_page: $per_page, page: $page) {
                results
            }
        }"#;
        let vars =
            json!({ "q": search_query, "type": query_type, "per_page": per_page, "page": page });
        let data = self.query(query, Some(vars)).await?;
        Ok(data["search"]["results"].clone())
    }

    // --- User profile ---

    pub async fn user_by_username(&self, username: &str) -> Result<Value> {
        let query = r#"query ($username: String!) {
            users(where: { username: { _eq: $username } }, limit: 1) {
                id username name bio location books_count
                followers_count followed_users_count
                cached_image
            }
        }"#;
        let vars = json!({ "username": username });
        let data = self.query(query, Some(vars)).await?;
        Ok(data["users"][0].clone())
    }

    pub async fn update_profile(&self, updates: Value) -> Result<Value> {
        let query = r#"mutation ($user: update_user_input!) {
            update_user(user: $user) { id username name bio location }
        }"#;
        let vars = json!({ "user": updates });
        self.query(query, Some(vars)).await
    }

    // --- Follows ---

    pub async fn follow_entity(&self, followable_id: i64, followable_type: &str) -> Result<Value> {
        let query = r#"mutation ($id: Int!, $type: String!) {
            insert_follow(followable_id: $id, followable_type: $type) { id }
        }"#;
        let vars = json!({ "id": followable_id, "type": followable_type });
        self.query(query, Some(vars)).await
    }

    pub async fn unfollow_entity(
        &self,
        followable_id: i64,
        followable_type: &str,
    ) -> Result<Value> {
        let query = r#"mutation ($id: Int!, $type: String!) {
            delete_follow(followable_id: $id, followable_type: $type) { id }
        }"#;
        let vars = json!({ "id": followable_id, "type": followable_type });
        self.query(query, Some(vars)).await
    }

    pub async fn follow_list(&self, list_id: i64) -> Result<Value> {
        let query = r#"mutation ($list_id: Int!) {
            upsert_followed_list(list_id: $list_id) { id }
        }"#;
        let vars = json!({ "list_id": list_id });
        self.query(query, Some(vars)).await
    }

    pub async fn unfollow_list(&self, list_id: i64) -> Result<Value> {
        let query = r#"mutation ($list_id: Int!) {
            delete_followed_list(list_id: $list_id) { id }
        }"#;
        let vars = json!({ "list_id": list_id });
        self.query(query, Some(vars)).await
    }

    pub async fn my_following(&self) -> Result<Value> {
        let query = r#"{ me { followed_users(order_by: { created_at: desc }) {
            id user { id username name }
        } } }"#;
        let data = self.query(query, None).await?;
        Ok(data["me"][0]["followed_users"].clone())
    }

    // --- Likes ---

    pub async fn like(&self, likeable_id: i64, likeable_type: &str) -> Result<Value> {
        let query = r#"mutation ($id: Int!, $type: String) {
            upsert_like(likeable_id: $id, likeable_type: $type) { id }
        }"#;
        let vars = json!({ "id": likeable_id, "type": likeable_type });
        self.query(query, Some(vars)).await
    }

    pub async fn unlike(&self, likeable_id: i64, likeable_type: &str) -> Result<Value> {
        let query = r#"mutation ($id: Int!, $type: String!) {
            delete_like(likeable_id: $id, likeable_type: $type) { id }
        }"#;
        let vars = json!({ "id": likeable_id, "type": likeable_type });
        self.query(query, Some(vars)).await
    }

    // --- Reading Journals ---

    pub async fn my_journals(&self, book_id: Option<i64>, limit: i32) -> Result<Value> {
        let query = r#"query ($limit: Int!, $where: reading_journals_bool_exp) {
            me {
                reading_journals(limit: $limit, order_by: {created_at: desc}, where: $where) {
                    id event entry action_at book_id edition_id
                    book { id title slug }
                }
            }
        }"#;

        let mut where_obj = json!({});
        if let Some(bid) = book_id {
            where_obj = json!({ "book_id": { "_eq": bid } });
        }

        let vars = json!({
            "limit": limit,
            "where": where_obj
        });

        let data = self.query(query, Some(vars)).await?;
        Ok(data["me"][0]["reading_journals"].clone())
    }

    pub async fn create_journal(
        &self,
        book_id: i64,
        event: &str,
        entry: Option<&str>,
        action_at: Option<&str>,
        edition_id: Option<i64>,
        privacy_setting_id: i32,
    ) -> Result<Value> {
        let query = r#"mutation ($object: ReadingJournalCreateType!) {
            insert_reading_journal(object: $object) { id event entry }
        }"#;
        let mut obj = json!({
            "book_id": book_id,
            "event": event,
            "privacy_setting_id": privacy_setting_id,
            "tags": []
        });
        if let Some(e) = entry {
            obj["entry"] = json!(e);
        }
        if let Some(d) = action_at {
            obj["action_at"] = json!(d);
        }
        if let Some(eid) = edition_id {
            obj["edition_id"] = json!(eid);
        }
        let vars = json!({ "object": obj });
        self.query(query, Some(vars)).await
    }

    pub async fn update_journal(&self, journal_id: i64, updates: Value) -> Result<Value> {
        let query = r#"mutation ($id: Int!, $object: ReadingJournalUpdateType!) {
            update_reading_journal(id: $id, object: $object) { id event entry }
        }"#;
        let vars = json!({ "id": journal_id, "object": updates });
        self.query(query, Some(vars)).await
    }

    pub async fn delete_journal(&self, journal_id: i64) -> Result<Value> {
        let query = r#"mutation ($id: Int!) {
            delete_reading_journal(id: $id) { id }
        }"#;
        let vars = json!({ "id": journal_id });
        self.query(query, Some(vars)).await
    }

    // --- Book reads (date tracking) ---

    pub async fn add_book_read(
        &self,
        user_book_id: i64,
        started_at: Option<&str>,
        finished_at: Option<&str>,
        edition_id: Option<i64>,
        progress: Option<f64>,
        progress_pages: Option<i32>,
    ) -> Result<Value> {
        let query = r#"mutation ($user_book_id: Int!, $user_book_read: DatesReadInput!) {
            insert_user_book_read(user_book_id: $user_book_id, user_book_read: $user_book_read) { id }
        }"#;
        let mut read = json!({});
        if let Some(s) = started_at {
            read["started_at"] = json!(s);
        }
        if let Some(f) = finished_at {
            read["finished_at"] = json!(f);
        }
        if let Some(e) = edition_id {
            read["edition_id"] = json!(e);
        }
        if let Some(p) = progress {
            read["progress"] = json!(p);
        }
        if let Some(pp) = progress_pages {
            read["progress_pages"] = json!(pp);
        }
        let vars = json!({ "user_book_id": user_book_id, "user_book_read": read });
        self.query(query, Some(vars)).await
    }

    pub async fn update_book_read(&self, read_id: i64, updates: Value) -> Result<Value> {
        let query = r#"mutation ($id: Int!, $object: DatesReadInput!) {
            update_user_book_read(id: $id, object: $object) { id }
        }"#;
        let vars = json!({ "id": read_id, "object": updates });
        self.query(query, Some(vars)).await
    }

    pub async fn delete_book_read(&self, read_id: i64) -> Result<Value> {
        let query = r#"mutation ($id: Int!) {
            delete_user_book_read(id: $id) { id }
        }"#;
        let vars = json!({ "id": read_id });
        self.query(query, Some(vars)).await
    }

    pub async fn book_reads(&self, book_id: i64) -> Result<Value> {
        let query = r#"query ($book_id: Int!) {
            me { user_books(where: { book_id: { _eq: $book_id } }, limit: 1) {
                id user_book_reads(order_by: { started_at: desc }) {
                    id started_at finished_at edition_id progress progress_pages
                }
            } }
        }"#;
        let vars = json!({ "book_id": book_id });
        let data = self.query(query, Some(vars)).await?;
        Ok(data["me"][0]["user_books"][0]["user_book_reads"].clone())
    }

    // --- Authors ---

    pub async fn author_by_id(&self, id: i64) -> Result<Value> {
        let query = r#"query ($id: Int!) {
            authors_by_pk(id: $id) {
                id name slug bio born_year death_year location
                books_count users_count cached_image
            }
        }"#;
        let vars = json!({ "id": id });
        let data = self.query(query, Some(vars)).await?;
        Ok(data["authors_by_pk"].clone())
    }

    // --- Series ---

    pub async fn series_by_id(&self, id: i64) -> Result<Value> {
        let query = r#"query ($id: Int!) {
            series_by_pk(id: $id) {
                id name slug description books_count primary_books_count is_completed
                book_series(order_by: { position: asc }) {
                    position book { id title slug release_year cached_contributors }
                }
            }
        }"#;
        let vars = json!({ "id": id });
        let data = self.query(query, Some(vars)).await?;
        Ok(data["series_by_pk"].clone())
    }

    // --- Editions ---

    pub async fn editions_by_book_id(&self, book_id: i64) -> Result<Value> {
        let query = r#"query ($book_id: Int!) {
            editions(where: { book_id: { _eq: $book_id } }) {
                id title edition_format pages release_date isbn_10 isbn_13
                publisher { name }
            }
        }"#;
        let vars = json!({ "book_id": book_id });
        let data = self.query(query, Some(vars)).await?;
        Ok(data["editions"].clone())
    }

    pub async fn edition_by_isbn(&self, isbn: &str) -> Result<Option<Value>> {
        let isbn_field = if isbn.len() == 10 {
            "isbn_10"
        } else {
            "isbn_13"
        };
        let query = format!(
            r#"query ($isbn: String!) {{
                editions(where: {{ {isbn_field}: {{ _eq: $isbn }} }}, limit: 1) {{
                    id title edition_format pages release_date isbn_10 isbn_13
                    publisher {{ name }}
                    book {{ id title slug cached_contributors }}
                }}
            }}"#
        );
        let vars = json!({ "isbn": isbn });
        let data = self.query(&query, Some(vars)).await?;
        let editions = data["editions"].as_array().cloned().unwrap_or_default();
        Ok(editions.first().cloned())
    }

    // --- Characters ---

    pub async fn character_by_id(&self, id: i64) -> Result<Value> {
        let query = r#"query ($id: bigint!) {
            characters_by_pk(id: $id) {
                id name slug biography gender_id is_lgbtq is_poc
                books_count cached_tags
                book_characters {
                    book { id title slug release_year cached_contributors }
                }
            }
        }"#;
        let vars = json!({ "id": id });
        let data = self.query(query, Some(vars)).await?;
        Ok(data["characters_by_pk"].clone())
    }

    pub async fn character_by_slug(&self, slug: &str) -> Result<Value> {
        let query = r#"query ($slug: String!) {
            characters(where: { slug: { _eq: $slug } }, limit: 1) {
                id name slug biography gender_id is_lgbtq is_poc
                books_count cached_tags
                book_characters {
                    book { id title slug release_year cached_contributors }
                }
            }
        }"#;
        let vars = json!({ "slug": slug });
        let data = self.query(query, Some(vars)).await?;
        Ok(data["characters"][0].clone())
    }

    // --- Tags ---

    pub async fn all_tags(
        &self,
        category_id: Option<i32>,
        limit: i32,
        offset: i32,
    ) -> Result<Value> {
        let query = r#"query ($limit: Int!, $offset: Int!, $where: tags_bool_exp) {
            tags(limit: $limit, offset: $offset, order_by: {count: desc}, where: $where) {
                id tag slug count tag_category_id
            }
        }"#;

        let mut where_obj = json!({});
        if let Some(cid) = category_id {
            where_obj = json!({ "tag_category_id": { "_eq": cid } });
        }

        let vars = json!({
            "limit": limit,
            "offset": offset,
            "where": where_obj
        });

        let data = self.query(query, Some(vars)).await?;
        Ok(data["tags"].clone())
    }

    // --- Notifications ---

    pub async fn my_notifications(&self, limit: i32, offset: i32) -> Result<Value> {
        let query = r#"query ($limit: Int!, $offset: Int!) {
            me {
                notifications(limit: $limit, offset: $offset, order_by: {created_at: desc}) {
                    id title description link created_at notification_type_id
                }
            }
        }"#;
        let vars = json!({ "limit": limit, "offset": offset });
        let data = self.query(query, Some(vars)).await?;
        Ok(data["me"][0]["notifications"].clone())
    }

    // --- Prompts ---

    pub async fn all_prompts(&self, limit: i32, offset: i32) -> Result<Value> {
        let query = format!(
            r#"{{ prompts(limit: {limit}, offset: {offset}, order_by: {{answers_count: desc}}) {{
                id question description answers_count slug
            }} }}"#
        );
        let data = self.query(&query, None).await?;
        Ok(data["prompts"].clone())
    }

    // --- Metadata Listings ---

    pub async fn all_platforms(&self) -> Result<Value> {
        let query = r#"{ platforms(order_by: {name: asc}) { id name } }"#;
        let data = self.query(query, None).await?;
        Ok(data["platforms"].clone())
    }

    pub async fn all_formats(&self) -> Result<Value> {
        let query = r#"{ reading_formats(order_by: {name: asc}) { id name } }"#;
        let data = self.query(query, None).await?;
        Ok(data["reading_formats"].clone())
    }

    pub async fn all_publishers(&self, limit: i32, offset: i32) -> Result<Value> {
        let query = r#"query ($limit: Int!, $offset: Int!) {
            publishers(limit: $limit, offset: $offset, order_by: {name: asc}) {
                id name
            }
        }"#;
        let vars = json!({ "limit": limit, "offset": offset });
        let data = self.query(query, Some(vars)).await?;
        Ok(data["publishers"].clone())
    }

    pub async fn upsert_tags(
        &self,
        id: i64,
        tag_names: Vec<String>,
        entity_type: &str,
    ) -> Result<Value> {
        let query = r#"mutation ($id: bigint!, $tags: [BasicTag]!, $type: String!) {
            upsert_tags(id: $id, tags: $tags, type: $type) {
                success
            }
        }"#;
        let tags = tag_names
            .into_iter()
            .map(|name| {
                json!({ "tag": name, "category": "Genre", "spoiler": false }) // Defaulting category for simplicity
            })
            .collect::<Vec<_>>();

        let vars = json!({
            "id": id,
            "tags": tags,
            "type": entity_type
        });
        self.query(query, Some(vars)).await
    }

    // --- Trending ---

    pub async fn trending_books(&self, limit: i32, offset: i32, days: i64) -> Result<Value> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let from = (chrono::Local::now() - chrono::Duration::days(days))
            .format("%Y-%m-%d")
            .to_string();

        let id_query = r#"query ($from: date!, $to: date!, $limit: Int!, $offset: Int!) {
            books_trending(from: $from, to: $to, limit: $limit, offset: $offset) {
                ids
            }
        }"#;
        let id_vars = json!({ "from": from, "to": today, "limit": limit, "offset": offset });
        let data = self.query(id_query, Some(id_vars)).await?;
        let ids: Vec<i64> = data["books_trending"]["ids"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
            .unwrap_or_default();

        if ids.is_empty() {
            return Ok(Value::Array(vec![]));
        }

        let query = r#"query ($ids: [bigint!]!) {
            books(where: { id: { _in: $ids } }) {
                id title slug rating users_count
                cached_contributors cached_image
            }
        }"#;
        let vars = json!({ "ids": ids });
        let data = self.query(query, Some(vars)).await?;
        Ok(data["books"].clone())
    }
}
