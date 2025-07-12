use std::collections::HashSet;
use serde::{Deserialize,Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Word {
    pub word: String,
    pub chars: HashSet<char>,
    pub is_pangram: bool,
}

impl std::hash::Hash for Word {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.word.hash(state)
    }
}

impl std::cmp::PartialEq for Word {
    fn eq(&self, other: &Self) -> bool {
        self.word == other.word
    }
}

impl std::cmp::Eq for Word {}

impl Word {
    pub fn new(word: &str, is_pangram: bool) -> Self {
        Self {
            word: word.to_owned(),
            is_pangram,
            chars: word.chars().collect(),
        }
    }

    pub fn score(&self) -> u32 {
        if self.word.len() == 4 {
            1
        } else {
            let pangram_boost = if self.is_pangram { 7 } else { 0 };
            self.word.len() as u32 + pangram_boost
        }
    }

    pub fn is_superset(&self, other: &Word) -> bool {
        self.chars.is_superset(&other.chars)
    }

    pub fn get(&self, idx: usize) -> Option<Letter> {
        self.word
            .get(idx..=idx)
            .and_then(|s| s.chars().nth(0))
            .map(Letter::new)
    }

    pub fn len(&self) -> usize {
        self.word.len()
    }

    pub fn letters(&self) -> HashSet<Letter> {
        self.word
            .split("")
            .filter_map(|s| s.chars().nth(0))
            .map(Letter::new)
            .collect()
    }

    pub fn contains(&self, letter: &Letter) -> bool {
        self.word
            .split("")
            .filter_map(|s| s.chars().nth(0))
            .any(|l| l == letter.0)
    }
}


#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Letter(pub char);

impl Letter {
    pub fn new(s: char) -> Self {
        Self(s)
    }
}

pub type ScoreBuckets = [(String, u32); 9];

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PuzzleConfig {
    pub score_buckets: ScoreBuckets,
    pub required_letter: Letter,
    pub other_letters: Vec<Letter>,
    pub valid_words: HashSet<Word>,
}

