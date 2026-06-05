use crate::components::nav_bar;
use maud::{Markup, html};

pub fn header() -> Markup {
    html! {
        header class="primary-container" {
            nav {
                // Espacio libre para empujar el menú al centro
                div class="max" {}

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
