use maud::{Markup, html};

pub fn footer() -> Markup {
    html! {
        footer class="center-align padding gray-text" {
            p { "Hecho en 🦀 & 🐧 con amor ^^" }
        }
    }
}
