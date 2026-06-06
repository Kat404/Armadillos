use maud::{Markup, html};

/// Genera la interfaz que muestra el operador militar activo en la sesión.
pub fn info_operador(
    operador_nombre: Option<&str>,
    operador_rango: Option<&str>,
    operador_seccion: Option<&str>,
    puede_administrar_belico: bool,
) -> Markup {
    html! {
        article class="border round medium-padding margin-bottom" {
            div class="row wrap align-center" {
                div class="max" {
                    @if let (Some(nombre), Some(rango), Some(seccion)) = (operador_nombre, operador_rango, operador_seccion) {
                        div class="row gap align-center" {
                            span class="chip label circle secondary" { "person" }
                            div {
                                p class="bold no-margin" { (rango) " — " (nombre) }
                                p class="caption no-margin text-secondary" { "Sección: " (seccion) }
                            }
                            @if puede_administrar_belico {
                                span class="chip success" {
                                    i { "verified" }
                                    span { "AUTORIZADO PARA MATERIAL DE GUERRA" }
                                }
                            } @else {
                                span class="chip error" {
                                    i { "block" }
                                    span { "NO AUTORIZADO PARA MATERIAL BÉLICO" }
                                }
                            }
                        }
                    } @else {
                        p class="bold text-error no-margin" { "Sin sesión militar activa en el sistema" }
                    }
                }
                div {
                    @if operador_nombre.is_some() {
                        form action="/logout" method="POST" class="no-margin" {
                            button type="submit" class="button error outline row gap align-center" {
                                i { "logout" }
                                span { "Cerrar Sesión" }
                            }
                        }
                    } @else {
                        a href="/login" class="button primary row gap align-center no-margin" {
                            i { "login" }
                            span { "Iniciar Sesión" }
                        }
                    }
                }
            }
        }
    }
}
