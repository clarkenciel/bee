pub(crate) mod words {
    use std::fmt::Display;

    pub(crate) trait AddWords {
        async fn add_words(&self, words: Vec<String>) -> Result<(), AddWordsError>;
    }

    pub(crate) mod pg {
        use super::AddWordsError;

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
}
