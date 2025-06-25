use std::{
    collections::{BTreeSet, HashSet},
    f32::consts::PI,
    fmt::Write as _,
    sync::Arc,
};

use codee::{Decoder, HybridEncoder};
use leptos::{attr::readonly, prelude::*};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

const WORDS: &str = include_str!("../assets/words.txt");

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let storage = web_sys::window()
        .expect("Failed to get window")
        .local_storage()
        .expect("Failed to get local storage")
        .expect("no local storage found");
    let storage_key = daydex().to_string();

    let data_key = format!("{}/data", storage_key);
    let data: Data = match storage.get(&data_key) {
        Ok(Some(data)) => match codee::string::JsonSerdeCodec::decode(&data) {
            Ok(data) => data,
            Err(e) => {
                leptos::logging::warn!("Stored data decoding failed: {}", e);
                let new_data = Data::default();
                storage
                    .set(
                        &data_key,
                        &codee::string::JsonSerdeCodec::encode_str(&new_data)
                            .expect("Failed to encode new data"),
                    )
                    .expect("Failed to store new data");
                new_data
            }
        },
        Ok(None) => {
            let new_data = Data::default();
            storage
                .set(
                    &data_key,
                    &codee::string::JsonSerdeCodec::encode_str(&new_data)
                        .expect("Failed to encode new data"),
                )
                .expect("Failed to store new data");
            new_data
        }
        Err(e) => panic!("Storage access failed {:?}", e),
    };
    let (score, set_score, _) = leptos_use::storage::use_local_storage::<
        u32,
        codee::string::JsonSerdeCodec,
    >(format!("{}/score", storage_key));
    let (submitted, set_submitted, _) = leptos_use::storage::use_local_storage::<
        BTreeSet<_>,
        codee::string::JsonSerdeCodec,
    >(format!("{}/submitted", storage_key));

    let Data {
        score_buckets,
        required_letter,
        available_letters,
        valid_words,
    } = data;
    let (word, set_word) = signal(String::new());
    let (_error, set_error) = signal(None);

    provide_context(set_word);

    leptos::logging::log!(
        "Words {}",
        valid_words
            .iter()
            .map(|w| w.word.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
    Effect::watch(
        move || word.get(),
        move |word, prev_word, _| {
            leptos::logging::log!("Word: {}; Prev: {:?}", word, prev_word);
        },
        false,
    );

    let available_letters_submit_check = available_letters.clone();

    let submit = move |e: web_sys::SubmitEvent| {
        e.prevent_default();

        let word = std::mem::take(&mut *set_word.write());
        if submitted.get().contains(&word) {
            return;
        }

        leptos::logging::log!("Checking {}", word);
        if !word.contains(required_letter.0) {
            *set_error.write() = Some(ValidationError::MissingRequiredLetter {
                letter: required_letter.0.clone(),
                candidate: word,
            });
            return;
        }

        let mut candidate = Word::new(&word, false);
        if !valid_words.contains(&candidate) {
            *set_error.write() = Some(ValidationError::InvalidWord { candidate: word });
            return;
        }
        candidate.is_pangram = available_letters_submit_check
            .iter()
            .all(|l| candidate.contains(l));

        *set_score.write() += candidate.score();
        set_submitted.write().insert(word);
    };

    let other_letters: Vec<Letter> = available_letters
        .iter()
        .filter(|l| **l != required_letter)
        .cloned()
        .collect();

    view! {
        <div class="container p-4">
            <div class="container flex flex-col w-full justify-between gap-1">
                <div class="self-start w-full">
                    <Score score=score buckets=score_buckets />
                </div>

                <button type="button" class="btn btn-ghost w-full" onclick="guessed.showModal()">
                    Guessed words
                </button>
                <dialog id="guessed" class="modal">
                    <section class="modal-box">
                        <h1>Guessed words</h1>
                        <ul>
                            <For
                                each=move || submitted.get()
                                key=|w| w.clone()
                                children=|word| {
                                    view! { <li>{word}</li> }
                                }
                            />
                        </ul>
                        <div class="modal-action">
                            <form method="dialog">
                                <button type="submit" class="btn">
                                    Close
                                </button>
                            </form>
                        </div>
                    </section>
                </dialog>
            </div>

            <div class="divider divider-secondary"></div>

            <form id="word-form" on:submit=submit class="w-full">
                <input
                    type="text"
                    class="input input-ghost input-xl w-full text-center"
                    bind:value=(word, set_word)
                    aria-label="word"
                    minlength=4
                    autofocus
                />
            </form>

            <LetterGrid required_letter=required_letter.clone() other_letters=other_letters />

            <div class="grid grid-cols-12">
                <button
                    type="button"
                    class="btn btn-warning col-start-2 col-span-4"
                    on:click=move |_| {
                        set_word.write().pop();
                    }
                >
                    delete
                </button>
                <button
                    type="submit"
                    form="word-form"
                    class="btn btn-primary col-start-8 col-span-4"
                >
                    submit
                </button>
            </div>
        </div>
    }
}

#[component]
fn Score(score: Signal<u32>, buckets: ScoreBuckets) -> impl IntoView {
    let max = buckets[8].1;
    let (buckets, _) = signal(buckets);
    let current_threshold = Signal::derive(move || {
        buckets
            .get()
            .iter()
            .rfind(|(_label, thresh)| score.get() >= *thresh)
            .cloned()
            .map(|(label, _score)| label)
            .unwrap_or_else(|| buckets.get()[8].0.clone())
    });

    view! {
        <div class="grid grid-cols-12 items-center w-full">
            <div aria-label="current level" class="font-bold col-span-3">
                {current_threshold}
            </div>
            <div
                class="col-span-9"
                role="progressbar"
                aria-valuenow=score
                aria-valuemax=max
                aria-label="score progress"
            >
                <div class="progress-segments">
                    <For
                        each=move || buckets.get()
                        key=|(label, _)| label.clone()
                        children=move |(label, score_threshold)| {
                            let current_threshold = Signal::derive(move || {
                                if label == current_threshold.get() {
                                    Some(score.get())
                                } else {
                                    None
                                }
                            });
                            let is_filled = move || score.get() >= score_threshold;

                            view! {
                                <div
                                    class="segment"
                                    class:filled=is_filled
                                    class:current=move || { current_threshold.get().is_some() }
                                >
                                    {current_threshold}
                                </div>
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}

/// Positions for hexes on the grid.
/// Assumes 40px radii with 20px gaps
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum HexPos {
    Center,
    North,
    South,
    NorthEast,
    SouthEast,
    NorthWest,
    SouthWest,
}

impl HexPos {
    fn points(&self) -> Vec<(f32, f32)> {
        self.into()
    }

    fn center(&self) -> (f32, f32) {
        self.into()
    }
}

/// Mapping of positions to coordinates used in SVG.
/// Assumes a 500x400 inverted-y "canvas"
impl From<&HexPos> for (f32, f32) {
    fn from(value: &HexPos) -> Self {
        match value {
            HexPos::Center => (250.0, 140.0),
            HexPos::North => (250.0, 60.0),
            HexPos::South => (250.0, 220.0),
            HexPos::NorthEast => (319.28, 100.0),
            HexPos::SouthEast => (319.28, 180.0),
            HexPos::NorthWest => (180.72, 100.0),
            HexPos::SouthWest => (180.72, 180.0),
        }
    }
}

impl From<&HexPos> for Vec<(f32, f32)> {
    fn from(value: &HexPos) -> Self {
        hex_points(&value.into())
    }
}

const HEX_ANGLES: &[f32] = &[0.0, 60.0, 120.0, 180.0, 240.0, 300.0];

const HEX_RADIUS: f32 = 45.0;

/// Points of a hexagon with radius 20 around a center point
/// x = cx + r * cos(angle)
/// y = cy + r * sin(angle)
fn hex_points((cx, cy): &(f32, f32)) -> Vec<(f32, f32)> {
    HEX_ANGLES
        .iter()
        .map(|angle| {
            (
                cx + HEX_RADIUS * (angle * (PI / 180.0)).cos(),
                cy + HEX_RADIUS * (angle * (PI / 180.0)).sin(),
            )
        })
        .collect()
}

#[component]
fn LetterHex(#[prop(name = "letter")] Letter(l): Letter, pos: HexPos) -> impl IntoView {
    let add_letter = use_context::<WriteSignal<String>>().expect("No word context provided");

    let (cx, cy) = pos.center();
    let points = pos
        .points()
        .into_iter()
        .try_fold(
            String::new(),
            |mut s, (px, py)| -> Result<String, std::fmt::Error> {
                write!(&mut s, "{},{} ", px, py)?;
                Ok(s)
            },
        )
        .expect("Hex pointstring build failed!");

    view! {
        <polygon
            points=points
            class="cursor-pointer stroke-neutral stroke-2 hover:fill-accent fill-info"
            on:click:target=move |e| {
                e.prevent_default();
                leptos::logging::log!("CLICKED LETTER {}", l);
                add_letter.write().push(l)
            }
        />
        <text
            class="hex-text font-extrabold uppercase cursor-pointer dark:text-base-content dark:hover:text-accent-content"
            x=cx - 10.0
            y=cy + 10.0
            pointer-events="none"
        >
            {l}
        </text>
    }
}

const OTHER_LETTER_POSITIONS: &[HexPos] = &[
    HexPos::North,
    HexPos::NorthEast,
    HexPos::SouthEast,
    HexPos::South,
    HexPos::SouthWest,
    HexPos::NorthWest,
];

#[component]
fn LetterGrid(required_letter: Letter, other_letters: Vec<Letter>) -> impl IntoView {
    let other_letters = other_letters
        .iter()
        .cloned()
        .zip(OTHER_LETTER_POSITIONS.iter().copied())
        .collect::<Vec<(Letter, HexPos)>>();

    view! {
        <div class="h-[300px] sm:h-auto overflow-hidden flex items-center justify-center">
            <svg
                class="w-full h-auto hex-container"
                viewBox="0 0 500 280"
                preserveAspectRatio="xMidYMid meet"
            >
                <LetterHex letter=required_letter pos=HexPos::Center />

                <For each=move || other_letters.clone() key=|hex| hex.clone() let((letter, pos))>
                    <LetterHex letter=letter pos=pos />
                </For>
            </svg>
        </div>
    }
}

type ScoreBuckets = [(String, u32); 9];

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct Data {
    score_buckets: ScoreBuckets,
    required_letter: Letter,
    available_letters: Vec<Letter>,
    valid_words: HashSet<Word>,
}

impl Default for Data {
    fn default() -> Self {
        Self::from_wordstr(&WORDS)
    }
}

impl Data {
    fn from_wordstr(word_str: &str) -> Self {
        let daydex = daydex();

        let mut valid_words = HashSet::new();
        let mut available_letters = Vec::with_capacity(7);
        let mut required_letter = '\0';
        let mut rng = rand::rngs::SmallRng::seed_from_u64(daydex);
        while !valid_words.iter().any(|w: &Word| w.is_pangram) {
            valid_words.clear();

            available_letters.clear();
            while available_letters.len() < 7 {
                let candidate = rng.random_range('a'..='z');
                if !available_letters.contains(&candidate) {
                    available_letters.push(candidate);
                }
            }

            required_letter = available_letters[0].clone();

            valid_words.extend(
                word_str
                    .lines()
                    .filter(|l| {
                        l.contains(required_letter)
                            && l.chars().all(|v| available_letters.contains(&v))
                    })
                    .map(|w: &str| {
                        let is_pangram = available_letters.iter().all(|l| w.contains(*l));
                        Word::new(w, is_pangram)
                    }),
            );
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

        Self {
            score_buckets,
            required_letter: Letter::new(required_letter),
            valid_words,
            available_letters: available_letters.into_iter().map(Letter::new).collect(),
        }
    }
}

fn daydex() -> u64 {
    let datetime = js_sys::Date::new_0();
    datetime.set_hours(0);
    datetime.set_minutes(0);
    datetime.set_seconds(0);
    datetime.set_milliseconds(0);
    leptos::logging::log!("datetime {:?}", datetime);
    let daydex = datetime.get_time() as u64;
    leptos::logging::log!("daydex {}", daydex);
    daydex
}

enum ValidationError {
    MissingRequiredLetter { letter: char, candidate: String },
    TooShort { candidate: String },
    InvalidWord { candidate: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Word {
    word: String,
    chars: HashSet<char>,
    is_pangram: bool,
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
    fn new(word: &str, is_pangram: bool) -> Self {
        Self {
            word: word.to_owned(),
            is_pangram,
            chars: word.chars().collect(),
        }
    }
}

impl Word {
    fn score(&self) -> u32 {
        if self.word.len() == 4 {
            1
        } else {
            self.word.len() as u32
        }
    }

    fn is_superset(&self, other: &Word) -> bool {
        self.chars.is_superset(&other.chars)
    }

    fn get(&self, idx: usize) -> Option<Letter> {
        self.word
            .get(idx..=idx)
            .and_then(|s| s.chars().nth(0))
            .map(Letter::new)
    }

    fn len(&self) -> usize {
        self.word.len()
    }

    fn letters(&self) -> HashSet<Letter> {
        self.word
            .split("")
            .filter_map(|s| s.chars().nth(0))
            .map(Letter::new)
            .collect()
    }

    fn contains(&self, letter: &Letter) -> bool {
        self.word
            .split("")
            .filter_map(|s| s.chars().nth(0))
            .any(|l| l == letter.0)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
struct Letter(char);

impl Letter {
    fn new(s: char) -> Self {
        Self(s)
    }
}
