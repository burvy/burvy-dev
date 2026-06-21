use leptos::prelude::*;

fn main() {
    leptos::prelude::mount_to_body(|| {
        view! {
            <main>
                <h1>"welcome to burvy's site"</h1>
                <Counter />
            </main>
        }
    });
}

#[component]
fn Counter() -> impl IntoView {
    let (count, set_count) = signal(0);
    view! {
        <button on:click=move |_| set_count.update(|n| *n += 1)>"clicked " {count} " times"</button>
    }
}
