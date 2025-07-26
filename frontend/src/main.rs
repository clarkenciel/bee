use std::{
    collections::{BTreeSet, HashSet},
    time::Duration,
};

use leptos::prelude::*;
use rand::SeedableRng;

use puzzle_config::{Letter, PuzzleConfig, ScoreBuckets, Word};

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let storage_key = day_64().to_string();

    let (score, set_score, _) = leptos_use::storage::use_local_storage::<
        u32,
        codee::string::JsonSerdeCodec,
    >(format!("{}/score", storage_key));
    provide_context((score, set_score));
    let (submitted, set_submitted, _) = leptos_use::storage::use_local_storage::<
        Vec<_>,
        codee::string::JsonSerdeCodec,
    >(format!("{}/submitted", storage_key));
    provide_context((submitted, set_submitted));

    let config = LocalResource::new(move || load());

    view! {
        <Suspense
            fallback=move || view! { <p>"Loading ..."</p> }
        >
        {move || Suspend::new(async move {
            match config.await {
                Ok(PuzzleConfig {
                score_buckets,
                required_letter,
                other_letters,
                valid_words,
            }) =>
            leptos::either::Either::Left(view! {
            <div class="container p-4 h-full">
                <div class="container flex flex-col w-full justify-between gap-1">
                    <div class="self-start w-full">
                        <Score score=score buckets=score_buckets />
                    </div>

                    <GuessedWords submitted />
                </div>

                <div class="divider divider-secondary"></div>

                <Board
                    required_letter=required_letter
                    other_letters=other_letters
                    valid_words=valid_words
                />
            </div>
            }),
            Err(AppError::ConfigLoadError(e)) => leptos::either::Either::Right( view! {
                <div>
                    <h1>Oopsie!</h1>
                    <p>{e}</p>
                    </div>
            })
        }
                                         })
        }
        </Suspense>
    }
}

#[component]
fn Board(
    required_letter: Letter,
    other_letters: Vec<Letter>,
    valid_words: HashSet<Word>,
) -> impl IntoView {
    let (valid_words, _) = signal(valid_words);
    let (required_letter, _) = signal(required_letter);
    let (other_letters, set_other_letters) = signal(other_letters);
    let (_, rng) = signal(rand::rngs::SmallRng::seed_from_u64(day_64()));

    let (word, set_word) = signal(String::new());
    provide_context(set_word);
    Effect::watch(
        move || word.get(),
        move |word, prev_word, _| {
            leptos::logging::log!("Word: {}; Prev: {:?}", word, prev_word);
        },
        false,
    );

    let (_score, set_score) =
        use_context::<(Signal<u32>, WriteSignal<u32>)>().expect("No writable score provided");
    let (submitted, set_submitted) =
        use_context::<(Signal<Vec<String>>, WriteSignal<Vec<String>>)>()
            .expect("No writable submittion list provided");
    let (set_error, error) = use_validation_errors();
    let submit = move |e: web_sys::SubmitEvent| {
        e.prevent_default();

        let word = std::mem::take(&mut *set_word.write());
        if word.len() < 4 {
            set_error.set(Some(ValidationError::TooShort));
            return;
        }

        if submitted.read().contains(&word) {
            set_error.set(Some(ValidationError::AlreadyGuessed));
            return;
        }

        leptos::logging::log!("Checking {}", word);
        if !word.contains(required_letter.read().0) {
            set_error.set(Some(ValidationError::MissingRequiredLetter));
            return;
        }

        if word.chars().any(|c| {
            !(required_letter.read().0 == c || other_letters.read().contains(&Letter::new(c)))
        }) {
            set_error.set(Some(ValidationError::BadLetters));
            return;
        }

        let mut candidate = Word::new(&word, false);
        if !valid_words.read().contains(&candidate) {
            set_error.set(Some(ValidationError::NotInList));
            return;
        }

        candidate.is_pangram = candidate.contains(&*required_letter.read())
            && other_letters.read().iter().all(|l| candidate.contains(l));

        *set_score.write() += candidate.score();
        set_submitted.write().push(word);
    };

    let shuffle_letters = move |_| {
        use rand::seq::SliceRandom;
        let rng = &mut *rng.write();
        set_other_letters.write().shuffle(rng);
    };

    view! {
        <div id="board">
            {error}
            <form id="word-form" on:submit=submit class="w-full h-auto">
                <input
                    type="text"
                    class="input input-ghost input-xl w-full text-center"
                    bind:value=(word, set_word)
                    aria-label="word"
                    minlength=4
                />
            </form>
            <LetterGrid required_letter=required_letter other_letters=other_letters />
            <div class="grid grid-cols-12 button-container join join-horizontal">
                <button
                    type="button"
                    class="btn btn-warning btn-outline join-item col-start-2 col-span-4"
                    on:click=move |_| {
                        set_word.write().pop();
                    }
                >
                    delete
                </button>
                <button
                    type="button"
                    aria-label="shuffle letters"
                    class="btn btn-accent btn-outline col-span-2 join-item grid justify-items-center"
                    on:click=shuffle_letters
                >
                    <ShuffleIcon />
                </button>
                <button
                    type="submit"
                    form="word-form"
                    class="btn btn-primary btn-outline join-item col-start-8 col-span-4"
                >
                    submit
                </button>
            </div>
        </div>
    }
}

fn use_validation_errors() -> (WriteSignal<Option<ValidationError>>, impl IntoView) {
    let (error, set_error) = signal(None);
    let message = move || {
        error.read().as_ref().map(|error| match error {
            ValidationError::BadLetters => "Bad letters",
            ValidationError::TooShort => "Too short",
            ValidationError::MissingRequiredLetter => "Missing center letter",
            ValidationError::AlreadyGuessed => "Already found",
            ValidationError::NotInList => "Not in word list",
        })
    };
    Effect::watch(
        move || error.get(),
        move |error, prev_error, _| {
            if error.is_some() && prev_error.flatten().is_none() {
                set_timeout(move || set_error.set(None), Duration::from_millis(1000))
            }
        },
        false,
    );

    (
        set_error,
        view! {
            <div
                aria-live="polite"
                class="alert alert-info text-2xl transition-opacity  duration-300"
                class=("opacity-100", move || error.read().is_some())
                class=("opacity-0", move || error.read().is_none())
            >
                {message}
            </div>
        },
    )
}

#[component]
fn ShuffleIcon() -> impl IntoView {
    view! {
        <svg
            width="24px"
            height="24px"
            stroke-width="1.5"
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
            aria-label="shuffle icon"
            class="fill-accent stroke-accent"
        >
            <path
                d="M22 6.99999C19 6.99999 13.5 6.99999 11.5 12.5C9.5 18 5 18 2 18"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
            ></path>
            <path
                d="M20 5C20 5 21.219 6.21895 22 7C21.219 7.78105 20 9 20 9"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
            ></path>
            <path
                d="M22 18C19 18 13.5 18 11.5 12.5C9.5 6.99999 5 7.00001 2 7"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
            ></path>
            <path
                d="M20 20C20 20 21.219 18.781 22 18C21.219 17.219 20 16 20 16"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
            ></path>
        </svg>
    }
}

#[cfg(not(debug_assertions))]
const PAGE_SIZE: usize = 10;

#[cfg(debug_assertions)]
const PAGE_SIZE: usize = 1;

#[component]
fn GuessedWords(#[prop(into)] submitted: Signal<Vec<String>>) -> impl IntoView {
    let (current_page, set_current_page) = signal(0);
    let submitted_alphabetically =
        Signal::derive(move || submitted.get().into_iter().collect::<BTreeSet<_>>());
    let pages = move || {
        submitted_alphabetically
            .read()
            .iter()
            .fold(vec![vec![]], |mut pages, word| {
                let page = pages.last_mut().unwrap();
                if page.len() >= PAGE_SIZE {
                    pages.push(vec![word.clone()])
                } else {
                    page.push(word.clone());
                }
                pages
            })
    };

    let latest_words = move || {
        submitted
            .read()
            .iter()
            .rev()
            .take(20)
            .cloned()
            .collect::<Vec<String>>()
    };

    view! {
        <div>
            <button
                type="button"
                class="btn btn-soft grid grid-cols-6 gap-2 w-full"
                onclick="guessed.showModal()"
            >
                <ul class="col-span-5 flex flex-row gap-4 overflow-y-hidden">
                    <For each=latest_words key=|s| s.clone() let(word)>
                        <li>{word}</li>
                    </For>
                </ul>
                <span class="col-span-1">. . .</span>
            </button>
            <dialog id="guessed" class="modal">
                <section class="modal-box">
                    <h1>Guessed words</h1>
                    <ul>
                        <For
                            each=move || pages()[*current_page.read()].clone()
                            key=|w| w.clone()
                            let(word)
                        >
                            <li>{word}</li>
                        </For>
                    </ul>
                    <div class="modal-action">
                        <button
                            type="button"
                            class="btn"
                            on:click=move |_| *set_current_page.write() -= 1
                            disabled=move || !(1..pages().len()).contains(&*current_page.read())
                        >
                            prev
                        </button>
                        <button
                            type="button"
                            class="btn"
                            on:click=move |_| *set_current_page.write() += 1
                            disabled=move || {
                                !(0..(pages().len() - 1)).contains(&*current_page.read())
                            }
                        >
                            next
                        </button>
                        <form method="dialog">
                            <button type="submit" class="btn btn-primary">
                                Close
                            </button>
                        </form>
                    </div>
                </section>
            </dialog>
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
        <div>
            <div
                class="grid grid-cols-12 items-center w-full cursor-pointer"
                onclick="scoreDetails.showModal()"
            >
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
            <dialog id="scoreDetails" class="modal">
                <section class="modal-box">
                    <h1 class="text-3xl">Rankings</h1>
                    <table class="table grid grid-cols-[1rm_auto_1vw_auto]">
                        <thead class="font-bold text-sm">
                            <tr>
                                <th></th>
                                <th>Rank</th>
                                <th></th>
                                <th>Minimum</th>
                            </tr>
                        </thead>

                        <For
                            each=move || buckets.get()
                            key=|(label, _)| label.clone()
                            children=move |(label, score_threshold)| {
                                let (label, _) = signal(label);
                                let current_threshold = Signal::derive(move || {
                                    if *label.read() == current_threshold.get() {
                                        Some(score.get())
                                    } else {
                                        None
                                    }
                                });

                                view! {
                                    <tr class=(
                                        ["font-bold"],
                                        move || { current_threshold.get().is_some() },
                                    )>
                                        <td>{current_threshold}</td>
                                        <td>{label}</td>
                                        <td></td>
                                        <td>{score_threshold}</td>
                                    </tr>
                                }
                            }
                        />
                    </table>
                    <div class="modal-action">
                        <form method="dialog">
                            <button type="submit" class="btn btn-primary">
                                close
                            </button>
                        </form>
                    </div>
                </section>
            </dialog>
        </div>
    }
}

#[component]
fn RequiredLetter(letter: ReadSignal<Letter>) -> impl IntoView {
    LetterHex(LetterHexProps {
        class: "letter required".to_owned(),
        letter,
    })
}

#[component]
fn OtherLetter(letter: ReadSignal<Letter>) -> impl IntoView {
    LetterHex(LetterHexProps {
        class: "letter other".to_owned(),
        letter,
    })
}

#[component]
fn LetterHex(class: String, letter: ReadSignal<Letter>) -> impl IntoView {
    let add_letter = use_context::<WriteSignal<String>>().expect("No word context provided");

    view! {
        <button
            type="button"
            class=class
            role="gridcell"
            aria-label=move || format!("letter {}", letter.read().0)
            on:click:target=move |e| {
                e.prevent_default();
                leptos::logging::log!("CLICKED LETTER {}", letter.read().0);
                add_letter.write().push(letter.read().0)
            }
            on:keyup:target=move |e| {
                e.prevent_default();
                if e.key() == "Enter" {
                    leptos::logging::log!("CLICKED LETTER {}", letter.read().0);
                    add_letter.write().push(letter.read().0)
                }
            }
        >
            <span>{move || letter.read().0}</span>
        </button>
    }
}

#[component]
fn LetterGrid(
    required_letter: ReadSignal<Letter>,
    other_letters: ReadSignal<Vec<Letter>>,
) -> impl IntoView {
    let board_ref = NodeRef::<leptos::html::Div>::new();
    
    Effect::new(move |_| {
        if let Some(board) = board_ref.get() {
            leptos::logging::log!("{:?}", board);
            let init = &web_sys::EventInit::new();
            init.set_bubbles(true);
            board.dispatch_event(
                &web_sys::Event::new_with_event_init_dict(
                    "bee:board-loaded",
                    &init,
                ).unwrap()).unwrap();
        }
    });

    view! {
        <div class="hex-container" aria-label="letter grid" role="grid" node_ref=board_ref >
            <RequiredLetter letter=required_letter />

            <For each=move || other_letters.get() key=|hex| hex.clone() let(letter)>
                <OtherLetter letter=signal(letter).0 />
            </For>
        </div>
    }
}

fn day_64() -> u64 {
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

#[derive(Clone)]
enum ValidationError {
    MissingRequiredLetter,
    TooShort,
    BadLetters,
    NotInList,
    AlreadyGuessed,
}

async fn load() -> Result<PuzzleConfig, AppError> {
    if let Some(config) = load_config_from_storage() {
        return Ok(config);
    }

    let fetched = fetch_config().await?;
    if let Err(e) = store_config(&fetched) {
        leptos::logging::error!("{}", e);
    }
    Ok(fetched)
}

#[derive(Debug, Clone)]
enum AppError {
    ConfigLoadError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, w: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::ConfigLoadError(cause) => {
                write!(w, "Failed to load puzzle config due to: {}", cause)
            }
        }
    }
}

impl std::error::Error for AppError {}

impl From<web_sys::wasm_bindgen::JsValue> for AppError {
    fn from(js_val: web_sys::wasm_bindgen::JsValue) -> Self {
        let js_err = js_sys::Error::from(js_val);
        Self::ConfigLoadError(js_err.message().as_string().unwrap())
    }
}

fn store_config(config: &PuzzleConfig) -> Result<(), AppError> {
    let storage = get_storage()?;
    let data =
        serde_json::to_string(config).map_err(|e| AppError::ConfigLoadError(e.to_string()))?;
    storage.set(&config_key(), &data).map_err(AppError::from)
}

fn load_config_from_storage() -> Option<PuzzleConfig> {
    let storage = get_storage().ok()?;
    let data = storage.get(&config_key()).ok().flatten()?;

    serde_json::from_str(&data).ok()
}

fn get_storage() -> Result<web_sys::Storage, AppError> {
    let window =
        web_sys::window().ok_or_else(|| AppError::ConfigLoadError("Window unavailable".into()))?;
    window
        .local_storage()
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::ConfigLoadError("Local storage unavailable".into()))
}

fn config_key() -> String {
    format!("puzzle-storage/{}", day_64())
}

async fn fetch_config() -> Result<PuzzleConfig, AppError> {
    let tz = get_current_tz()?;
    let resp = gloo_net::http::Request::get("/api/puzzle/daily/config")
        .query([("tz", tz)])
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|e| AppError::ConfigLoadError(e.to_string()))?;

    let json = resp.json().await;
    json.map_err(|e| AppError::ConfigLoadError(e.to_string()))
}

fn get_current_tz() -> Result<String, AppError> {
    let date = js_sys::Date::new_0();
    let minutes_to_utc = date.get_timezone_offset();
    let is_behind = minutes_to_utc.is_sign_positive();
    let offset_hours = (minutes_to_utc / 60.0).floor() as u64;
    let offset_minutes = minutes_to_utc as u64 % 60;

    Ok(format!(
        "{}{:02}:{:02}",
        if is_behind { "-" } else { "+" },
        offset_hours,
        offset_minutes
    ))
}
