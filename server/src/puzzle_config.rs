use std::sync::Arc;
use std::collections::HashSet;

use serde::Serialize;
use chrono::{DateTime, Duration, FixedOffset, Timelike, Utc};
use dashmap::DashMap;
use puzzle_config::{Letter, PuzzleConfig, Word};
use rand::{Rng, SeedableRng};

struct CachedConfig {
    config: PuzzleConfig,
    ttl: DateTime<FixedOffset>,
}

pub struct ConfigHandle<'a>(
    dashmap::mapref::one::MappedRef<'a, FixedOffset, CachedConfig, PuzzleConfig>,
);

impl<'a> std::ops::Deref for ConfigHandle<'a> {
    type Target = PuzzleConfig;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Serialize for ConfigHandle<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (**self).serialize(serializer)
    }
}

#[derive(Clone)]
pub struct ConfigProvider {
    cache: Arc<DashMap<FixedOffset, CachedConfig>>,
    pool: sqlx::PgPool,
}

impl ConfigProvider {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            pool,
        }
    }

    pub async fn get_config<'cache>(&'cache self, tz: &FixedOffset) -> Result<ConfigHandle<'cache>, Error> {
        let now = Utc::now().with_timezone(tz);
        if let Some(cached) = self.cache.get(tz)
            && cached.ttl >= now
        {
            return Ok(ConfigHandle(cached.map(|cached| &cached.config)));
        }

        let ttl = next_midnight(&now);
        let config = self.fetch().await?;
        Ok(ConfigHandle(
            self.cache
                .entry(tz.clone())
                .insert_entry(CachedConfig { config, ttl })
                .into_ref()
                .downgrade()
                .map(|cached| &cached.config)
        ))
    }

    async fn fetch(&self) -> Result<PuzzleConfig, Error> {
        let mut conn = self.pool.acquire().await.map_err(|e| Error::DbError(Box::new(e)))?;
        let mut rng = rand::rngs::StdRng::seed_from_u64(day_64());
        let mut letter_mask = 0i32;
        loop {
            let required_char = rng.random_range('a'..='z');
            let required_mask = words::letters::bitmask(&required_char);
            for _ in 0..6 {
                let letter = if rng.random_bool(0.5) {
                    rng.random_range('a'..required_char)
                } else {
                    rng.random_range(((required_char as u8 + 1) as char)..='z')
                };
                letter_mask |= words::letters::bitmask(&letter);
            };

            let words = sqlx::query_as!(
                WordRow,
                r#"select word, letter_mask & $1 = $1 as "is_pangram!"
                from words
                where letter_mask & $1 = letter_mask
                "#r,
                letter_mask | required_mask,
            )
                .fetch_all(&mut *conn)
                .await
                .map_err(|e| Error::DbError(Box::new(e)))?;

            if words.len() > 0 {
                let valid_words: HashSet<_> = words.into_iter().map(|w| Word::new(&w.word, w.is_pangram)).collect();
                let max_score = valid_words.iter().map(|w| w.score()).sum::<u32>() as f32;
                let score_buckets = [
                    ("Beginner".to_owned(), (max_score * 0.0).trunc() as u32),
                    ("Good Start".to_owned(), (max_score * 0.02).trunc() as u32),
                    ("Moving Up".to_owned(), (max_score * 0.05).trunc() as u32),
                    ("Good".to_owned(), (max_score * 0.08).trunc() as u32),
                    ("Solid".to_owned(), (max_score * 0.15).trunc() as u32),
                    ("Nice".to_owned(), (max_score * 0.25).trunc() as u32),
                    ("Great".to_owned(), (max_score * 0.4).trunc() as u32),
                    ("Amazing".to_owned(), (max_score * 0.5).trunc() as u32),
                    ("Genius".to_owned(), (max_score * 0.7).trunc() as u32),
                ];
                return Ok(PuzzleConfig {
                    valid_words,
                    score_buckets,
                    required_letter: Letter::new(words::letters::from_bitmask(&required_mask)),
                    other_letters: words::vec_from_bitmask(&letter_mask).into_iter().map(|l| Letter::new(l)).collect(),
                })
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct WordRow {
    word: String,
    is_pangram: bool,
}

#[derive(Debug)]
pub enum Error {
    DbError(Box<dyn std::error::Error>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::DbError(cause) => write!(f, "Failed to load puzzle config from database: {}", cause),
        }
    }
}

impl std::error::Error for Error {}

// TODO: make this timezone aware using browser TZ
fn next_midnight<Tz: chrono::TimeZone>(now: &DateTime<Tz>) -> DateTime<Tz> {
    (now.clone() + Duration::hours(24))
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap()
}

// TODO: make this timezone aware using browser TZ
fn day_64() -> u64 {
    Utc::now().timestamp() as u64
}
