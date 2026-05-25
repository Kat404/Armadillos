use maud::{Markup, html};

pub fn navbar() -> Markup {
    html! {
        a href="/" class="button transparent" {
            i { "home" }
            span class="m l" { "Inicio" }
        }
        a href="/about" class="button transparent" {
            i { "info" }
            span class="m l" { "Acerca de" }
        }
        a href="/soldiers" class="button transparent" {
            i { "shield" }
            span class="m l" { "Soldados" }
        }
    }
}
