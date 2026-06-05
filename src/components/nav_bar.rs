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
        a href="/soldados" class="button transparent" {
            i { "shield" }
            span class="m l" { "Soldados" }
        }
        a href="/inventario" class="button transparent" {
            i { "inventory_2" }
            span class="m l" { "Inventario" }
        }
        a href="/asignaciones" class="button transparent" {
            i { "assignment" }
            span class="m l" { "Asignaciones" }
        }
    }
}
