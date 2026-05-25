use crate::components::nav_bar;
use maud::{Markup, html};

pub fn header(title: &str) -> Markup {
    html! {
        header class="primary-container" {
            nav {
                h5 class="m l" {
                    (title)
                }
                h6 class="s" {
                    (title)
                }

                // Menú de navegación integrado
                (nav_bar::navbar())

                // Espacio libre para empujar el alternador de tema a la derecha
                div class="max" {}

                button class="circle transparent" id="theme-toggle" {
                    i { "dark_mode" }
                }
            }
        }
    }
}
