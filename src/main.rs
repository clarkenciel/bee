use leptos::prelude::*;

fn main() {
    println!("Hello, world!");
    leptos::mount::mount_to_body(|| view! {
        <p>Hello, world!</p>
    });
}
