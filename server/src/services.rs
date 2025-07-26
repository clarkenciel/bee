pub(crate) mod words {
    use std::fmt::Display;

    pub(crate) trait AddWords {
        async fn add_words(&self, words: Vec<String>) -> Result<(), AddWordsError>;
    }

    #[derive(Debug)]
    pub(crate) enum AddWordsError {
        DbError(Box<dyn std::error::Error>),
    }

    impl Display for AddWordsError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                AddWordsError::DbError(error) => {
                    write!(f, "Failed to add words due to database error: {}", error)
                }
            }
        }
    }

    impl std::error::Error for AddWordsError {}

    pub(crate) trait RemoveWords {
        async fn remove_words(&self, words: &[String]) -> Result<(), RemoveWordsError>;
    }

    #[derive(Debug)]
    pub(crate) enum RemoveWordsError {
        DbError(Box<dyn std::error::Error>),
    }

    impl Display for RemoveWordsError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                RemoveWordsError::DbError(error) => {
                    write!(f, "Failed to remove words due to database error: {}", error)
                }
            }
        }
    }

    pub(crate) trait SearchWords {
        async fn search(&self, query: &str) -> Result<SearchResult, SearchWordsError>;
    }

    type SearchResult = Vec<String>;

    #[derive(Debug)]
    pub(crate) enum SearchWordsError {
        DBError(Box<dyn std::error::Error>),
    }

    impl Display for SearchWordsError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::DBError(e) => write!(f, "Failed to search words due to db error: {}", e),
            }
        }
    }

    impl std::error::Error for SearchWordsError {}

    pub(crate) trait ListWords {
        async fn list(
            &self,
            cursor: &ListCursor,
            limit: Option<usize>,
        ) -> Result<ListedWords, ListWordsError>;
    }

    #[derive(Debug)]
    pub(crate) struct ListedWords {
        pub(crate) words: Vec<Word>,
        pub(crate) next_page: Option<ListCursor>,
    }

    #[derive(Debug)]
    pub(crate) struct Word {
        pub(crate) text: String,
        pub(crate) cursor: ListCursor,
    }

    #[derive(Debug)]
    pub(crate) struct ListCursor {
        pub(crate) after: String,
    }

    impl std::default::Default for ListCursor {
        fn default() -> Self {
            Self {
                after: "".to_owned(),
            }
        }
    }

    #[derive(Debug)]
    pub(crate) enum ListWordsError {
        DBError(Box<dyn std::error::Error>),
    }

    impl Display for ListWordsError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::DBError(e) => write!(f, "Failed to list words due to db error: {}", e),
            }
        }
    }

    impl std::error::Error for ListWordsError {}

    pub(crate) mod pg {
        use super::{AddWordsError, RemoveWordsError};

        #[derive(Clone)]
        pub(crate) struct AddWords(pub(crate) sqlx::PgPool);

        impl super::AddWords for AddWords {
            async fn add_words(&self, words: Vec<String>) -> Result<(), super::AddWordsError> {
                let mut builder =
                    sqlx::QueryBuilder::new("insert into words (word, letter_mask, length) ");
                builder.push_values(words, |mut b, word| {
                    let mask = words::bitmask(&word);
                    let length = word.len();
                    b.push_bind(word).push_bind(mask).push_bind(length as i32);
                });
                builder.push("on conflict do nothing");

                let mut conn = self
                    .0
                    .acquire()
                    .await
                    .map_err(|e| AddWordsError::DbError(Box::new(e)))?;
                builder
                    .build()
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AddWordsError::DbError(Box::new(e)))
                    .map(|_| ())
            }
        }

        #[derive(Clone)]
        pub(crate) struct RemoveWords(pub(crate) sqlx::PgPool);

        impl super::RemoveWords for RemoveWords {
            async fn remove_words(&self, words: &[String]) -> Result<(), RemoveWordsError> {
                let mut conn = self
                    .0
                    .acquire()
                    .await
                    .map_err(|e| RemoveWordsError::DbError(Box::new(e)))?;

                sqlx::query!(
                    "delete from words where word in (select * from unnest($1::text[]))",
                    words
                )
                .execute(&mut *conn)
                .await
                .map_err(|e| RemoveWordsError::DbError(Box::new(e)))
                .map(|_| ())
            }
        }

        #[derive(Clone)]
        pub(crate) struct SearchWords(pub(crate) sqlx::PgPool);

        impl super::SearchWords for SearchWords {
            async fn search(
                &self,
                query: &str,
            ) -> Result<super::SearchResult, super::SearchWordsError> {
                let mut conn = self
                    .0
                    .acquire()
                    .await
                    .map_err(|e| super::SearchWordsError::DBError(Box::new(e)))?;

                let result = sqlx::query_as!(
                    SearchResult,
                    r#"select word, levenshtein($1, word, 1, 2, 2) as "score!"
                    from words
                    order by "score!" asc
                    limit 15"#,
                    query
                )
                .fetch_all(&mut *conn)
                .await
                .map_err(|e| super::SearchWordsError::DBError(Box::new(e)))?;

                Ok(result.into_iter().map(|w| w.word).collect())
            }
        }

        #[derive(sqlx::FromRow)]
        struct SearchResult {
            word: String,
            score: i32,
        }

        #[derive(Clone)]
        pub(crate) struct ListWords(pub(crate) sqlx::PgPool);

        impl super::ListWords for ListWords {
            async fn list(
                &self,
                cursor: &super::ListCursor,
                limit: Option<usize>,
            ) -> Result<super::ListedWords, super::ListWordsError> {
                let mut conn = self
                    .0
                    .acquire()
                    .await
                    .map_err(|e| super::ListWordsError::DBError(Box::new(e)))?;

                let limit = limit.unwrap_or(200);
                let results = sqlx::query_as!(
                    ListedWord,
                    r#"
                         select word from words
                         where word > $1
                         limit $2
                     "#,
                    cursor.after,
                    (limit + 1) as i32
                )
                .fetch_all(&mut *conn)
                .await
                .map_err(|e| super::ListWordsError::DBError(Box::new(e)))?;

                let next_page = if results.len() > limit {
                    Some(super::ListCursor {
                        after: results[results.len() - 1].word.clone(),
                    })
                } else {
                    None
                };
                Ok(super::ListedWords {
                    words: results
                        .into_iter()
                        .map(|w| super::Word {
                            text: w.word.clone(),
                            cursor: super::ListCursor { after: w.word },
                        })
                        .collect(),
                    next_page,
                })
            }
        }

        #[derive(sqlx::FromRow)]
        struct ListedWord {
            word: String,
        }
    }
}
