use std::collections::HashSet;
use std::sync::Arc;

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
}

impl ConfigProvider {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    pub fn get_config<'cache>(&'cache self, tz: &FixedOffset) -> ConfigHandle<'cache> {
        let now = Utc::now().with_timezone(tz);
        if let Some(cached) = self.cache.get(tz)
            && cached.ttl >= now
        {
            return ConfigHandle(cached.map(|cached| &cached.config));
        }

        let ttl = next_midnight(&now);
        let config = from_wordstr(WORDS);
        ConfigHandle(
        self.cache
            .entry(tz.clone())
            .insert_entry(CachedConfig { config, ttl })
            .into_ref()
            .downgrade()
            .map(|cached| &cached.config))
    }
}

const WORDS: &str = include_str!("../data/words.txt");

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

pub fn from_wordstr(word_str: &str) -> PuzzleConfig {
    let daydex = day_64();

    let mut rng = rand::rngs::StdRng::seed_from_u64(daydex);
    let mut valid_words = HashSet::new();
    let mut required_letter = '\0';
    let mut other_letters = Vec::with_capacity(6);
    let mut all_letters = Vec::with_capacity(26);
    let mut available_letters = HashSet::with_capacity(7);
    let mut word_letters = HashSet::with_capacity(64);
    while !valid_words.iter().any(|w: &Word| w.is_pangram) {
        valid_words.clear();
        other_letters.clear();
        available_letters.clear();
        all_letters.clear();
        all_letters.extend('a'..'s');
        all_letters.extend('t'..'z');

        let required_idx = rng.random_range(0..all_letters.len());
        required_letter = all_letters[required_idx];
        available_letters.insert(required_letter);
        all_letters.remove(required_idx);

        for _ in 0..6 {
            let candidate_idx = rng.random_range(0..all_letters.len());
            let candidate = all_letters[candidate_idx];
            other_letters.push(candidate);
            available_letters.insert(candidate);
            all_letters.remove(candidate_idx);
        }

        valid_words.extend(word_str.lines().filter_map(|l| {
            word_letters.extend(l.chars());
            let maybe_word = match (
                word_letters.is_subset(&available_letters),
                available_letters.is_subset(&word_letters),
            ) {
                (true, true) => Some(Word::new(l, true)),
                (false, _) => None,
                _ => Some(Word::new(l, false)),
            };
            word_letters.clear();
            maybe_word
        }));
    }

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

    PuzzleConfig {
        score_buckets,
        required_letter: Letter::new(required_letter),
        valid_words,
        other_letters: other_letters.into_iter().map(Letter::new).collect(),
    }
}
