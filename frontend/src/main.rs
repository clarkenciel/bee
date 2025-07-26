use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

mod game;
mod management;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| "Not found">
                <Route path=path!("/") view=game::Game />
                <Route path=path!("/manage/words") view=management::Management />
            </Routes>
        </Router>
    }
}
