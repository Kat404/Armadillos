use maud::{Markup, html};

/// Componente que renderiza un indicador visual (badge/chip)
/// del nivel de acceso ABAC militar del operador activo.
pub fn badge_nivel_acceso(puede_administrar_belico: bool) -> Markup {
    html! {
        @if puede_administrar_belico {
            span class="chip success border extra-padding row gap align-center" style="border-radius: 8px;" {
                i { "verified" }
                span class="bold" { "AUTORIZADO PARA MATERIAL DE GUERRA" }
            }
        } @else {
            span class="chip error border extra-padding row gap align-center" style="border-radius: 8px;" {
                i { "block" }
                span class="bold" { "NO AUTORIZADO PARA MATERIAL BÉLICO" }
            }
        }
    }
}
