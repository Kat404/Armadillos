use maud::{Markup, html};

pub fn navbar() -> Markup {
    html! {
        nav {
            a href="/" { "Inicio" }
            a href="/about" { "Acerca de" }
            a href="/soldiers" { "Soldados" }
        }
    }
}
