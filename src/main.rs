use std::{collections::{BTreeSet, HashSet}, f32::consts::PI, fmt::Write as _};

use leptos::prelude::*;
use rand::{Rng, SeedableRng};

const WORDS: &str = include_str!("../assets/words.txt");

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let Data {
        max_score,
        required_letter,
        available_letters,
        valid_words,
    } = Data::from_wordstr(&WORDS);
    let max_score = max_score.clone();
    let (word, set_word) = signal(String::new());
    let (score, set_score) = signal(0);
    let (submitted, set_submitted) = signal(BTreeSet::new());
    let (_error, set_error) = signal(None);

    provide_context(set_word);

    leptos::logging::log!(
        "Words {}",
        valid_words
            .iter()
            .map(|w| w.word)
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
            <div class="container flex flex-row w-full justify-between gap-1">
                <div class="self-start w-full">
                    <label class="flex flex-row gap-1 text-2xl">
                        {score}
                        <progress class="progress progress-accent h-full self-end" max=max_score value=score />
                    </label>
                </div>

                <button type="button" class="btn btn-info self-end" onclick="guessed.showModal()">
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
                    class="input input-primary input-xl w-full"
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
            class="font-extrabold text-2xl uppercase cursor-pointer dark:text-base-content dark:hover:text-accent-content"
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
        <svg class="w-full h-auto" viewBox="0 0 500 280" preserveAspectRatio="xMidYMid meet">
            <LetterHex letter=required_letter pos=HexPos::Center />

            <For each=move || other_letters.clone() key=|hex| hex.clone() let((letter, pos))>
                <LetterHex letter=letter pos=pos />
            </For>
        </svg>
    }
}

#[derive(Debug, Clone)]
struct Data<'a> {
    max_score: u32,
    required_letter: Letter,
    available_letters: Vec<Letter>,
    valid_words: HashSet<Word<'a>>,
}

impl<'d> Data<'d> {
    fn from_wordstr<'words>(word_str: &'words str) -> Self
    where
        'words: 'd,
    {
        let datetime = js_sys::Date::new_0();
        datetime.set_hours(0);
        datetime.set_minutes(0);
        datetime.set_seconds(0);
        datetime.set_milliseconds(0);
        leptos::logging::log!("datetime {:?}", datetime);
        let daydex = datetime.get_time() as u64;
        leptos::logging::log!("daydex {}", daydex);

        let mut valid_words = HashSet::new();
        let mut available_letters = Vec::with_capacity(7);
        let mut required_letter = '\0';
        let mut rng = rand::rngs::SmallRng::seed_from_u64(daydex);
        while !valid_words.iter().any(|w: &Word<'_>| w.is_pangram) {
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

        let max_score = valid_words.iter().map(|w| w.score()).sum();

        Self {
            max_score,
            required_letter: Letter::new(required_letter),
            valid_words,
            available_letters: available_letters.into_iter().map(Letter::new).collect(),
        }
    }
}

enum ValidationError {
    MissingRequiredLetter { letter: char, candidate: String },
    TooShort { candidate: String },
    InvalidWord { candidate: String },
}

#[derive(Debug, Clone)]
struct Word<'a> {
    word: &'a str,
    chars: HashSet<char>,
    is_pangram: bool,
}

impl std::hash::Hash for Word<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.word.hash(state)
    }
}

impl std::cmp::PartialEq for Word<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.word == other.word
    }
}

impl std::cmp::Eq for Word<'_> {}

impl<'a> Word<'a> {
    fn new<'w>(word: &'w str, is_pangram: bool) -> Self
    where
        'w: 'a,
    {
        Self {
            word,
            is_pangram,
            chars: word.chars().collect(),
        }
    }
}

impl Word<'_> {
    fn score(&self) -> u32 {
        if self.word.len() == 4 {
            1
        } else {
            self.word.len() as u32
        }
    }

    fn is_superset(&self, other: &Word<'_>) -> bool {
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Letter(char);

impl Letter {
    fn new(s: char) -> Self {
        Self(s)
    }
}
