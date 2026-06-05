use maud::{Markup, html};

/// Genera la interfaz de simulación de usuario/operador activo para el control ABAC.
pub fn selector_operador(
    operador_id: Option<i64>,
    operador_nombre: Option<String>,
    operador_rango: Option<String>,
    operador_seccion: Option<String>,
    puede_administrar_belico: bool,
    soldados_opciones: &[(i64, String)],
    redir_path: &str,
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
                        p class="bold text-error no-margin" { "Sin operador simulado activo en el sistema" }
                    }
                }
                div {
                    form action="/simular_operador" method="POST" class="row gap align-center no-margin" {
                        input type="hidden" name="redir_path" value=(redir_path);
                        div class="field label border small no-margin" style="min-width: 240px;" {
                            select name="operador_id" onchange="this.form.submit()" {
                                @for &(s_id, ref label) in soldados_opciones {
                                    option value=(s_id) selected?[operador_id == Some(s_id)] {
                                        (label)
                                    }
                                }
                            }
                            label { "Cambiar Operador Activo" }
                        }
                    }
                }
            }
        }
    }
}
