use leptos::prelude::*;
use leptos_router::{
    components::Form,
    hooks::use_query,
    params::{Params, ParamsError},
};
use serde::Deserialize;

#[component]
pub fn Management() -> impl IntoView {
    let search_term = use_query::<WordSearch>();
    let words = LocalResource::new(move || {
        let search_term = search_term.get();
        leptos::logging::debug_warn!("search term: {:?}", search_term);
        search_words(search_term)
    });

    view! {
        <main class="container">
            <Search />
            <Suspense fallback=|| "Loading...">
                {move || Suspend::new(async move {
                    let words = words.await.unwrap_or_default();
                    view! {
                        <WordList words />
                    }
                })}
            </Suspense>
        </main>
    }
}

#[derive(Debug, PartialEq, Params, Clone)]
struct WordSearch {
    q: Option<String>,
}

#[component]
fn Search() -> impl IntoView {
    view! {
        <div id="word-search">
            <Form method="GET" action="/manage/words">
                <input
                    type="search"
                    name="q"
                    aria-label="Search words"
                    placeholder="Search words..."
                    oninput="this.form.requestSubmit()"
                />
            </Form>
        </div>
    }
}

#[component]
fn WordList(words: Vec<String>) -> impl IntoView {
    view! {
        <table>
            <thead>
                <tr>
                    <th scope="col">word</th>
                </tr>
            </thead>

            <For
                each=move || words.clone()
                key=|w| w.clone()
                let(word)
            >
                <tr><th scope="row">{word}</th></tr>
            </For>
        </table>
    }
}

async fn search_words(term: Result<WordSearch, ParamsError>) -> Option<Vec<String>> {
    if let Some(term) = term.ok()?.q
        && term != ""
    {
        let resp = gloo_net::http::Request::get("/api/words/search")
            .query([("q", term)])
            .header("accept", "application/json")
            .send()
            .await
            .ok()?;
        let json = resp.json::<SearchResponse>().await.ok()?;

        Some(json.words)
    } else {
        let resp = gloo_net::http::Request::get("/api/words")
            .header("accept", "application/json")
            .send()
            .await
            .ok()?;
        let json = resp.json::<SearchResponse>().await.ok()?;

        Some(json.words)
    }
}

#[derive(Deserialize, Clone)]
struct SearchResponse {
    words: Vec<String>,
}

enum SearchError {
    Fetch(String),
}
