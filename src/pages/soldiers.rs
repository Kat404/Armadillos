use crate::AppState;
use crate::layouts::main_layout::{PageContext, layout}; // Importamos nuestro Layout
use axum::{Form, extract::State};
use maud::{Markup, html};
use serde::Deserialize;

// Estructura para deserializar el formulario que envía HTMX
#[derive(Deserialize)]
pub struct NuevoSoldado {
    pub nombre: String,
    pub rango: String,
    pub estado: String,
}

// Estructura interna de datos
pub struct Soldado {
    pub id: i64,
    pub nombre: String,
    pub rango: String,
    pub estado: String,
}

// Función auxiliar para obtener soldados de la BD
async fn obtener_soldados(state: &AppState) -> Result<Vec<Soldado>, String> {
    let conn = state.db.connect().map_err(|e| e.to_string())?;
    let mut rows = conn
        .query("SELECT id, nombre, rango, estado FROM soldados", ())
        .await
        .map_err(|e| e.to_string())?;
    let mut lista = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        lista.push(Soldado {
            id: row.get(0).map_err(|e| e.to_string())?,
            nombre: row.get(1).map_err(|e| e.to_string())?,
            rango: row.get(2).map_err(|e| e.to_string())?,
            estado: row.get(3).map_err(|e| e.to_string())?,
        });
    }
    Ok(lista)
}

// Componente para renderizar las filas de la tabla de 'soldados'
fn render_filas_soldados(soldados: &[Soldado]) -> Markup {
    html! {
        @for s in soldados {
            tr {
                td { (s.id) }
                td { (s.nombre) }
                td { (s.rango) }
                td {
                    span class={
                        "chip "
                        @if s.estado == "Activo" { "primary-container" }
                        @else if s.estado == "Licencia" { "tertiary-container" }
                        @else { "error-container" }
                    } {
                        (s.estado)
                    }
                }
            }
        }
    }
}

// Handler de Página
pub async fn pagina_soldiers(State(state): State<AppState>) -> Markup {
    let ctx = PageContext::new(
        "Armadillos - Personal Militar",
        "Administración de Soldados",
        "Armadillos - Página para añadición y visualización de soldados dentro del sistema",
    );
    let soldados = obtener_soldados(&state).await.unwrap_or_default();

    layout(
        &ctx,
        html! {
            h4 class="medium-margin" {
                "Panel de Control de Soldados"
            }
            div class="grid" {
                // Columna Izquierda - Formulario (12/12 en móvil, 4/12 en tablet/escritorio)
                div class="s12 m4" {
                    article class="border round medium-padding" {
                        h5 class="medium-margin" {
                            "Incorporar Soldado"
                        }
                        form hx-post="/soldiers" hx-target="#tabla-soldados" hx-swap="innerHTML" {
                            div class="field label border" {
                                input type="text" id="nombre" name="nombre" placeholder=" " required;
                                label for="nombre" { "Nombre Completo" }
                            }

                            div class="field label border" {
                                select id="rango" name="rango" required {
                                    option value="Soldado" { "Soldado" }
                                    option value="Cabo" { "Cabo" }
                                    option value="Sargento" { "Sargento" }
                                    option value="Teniente" { "Teniente" }
                                    option value="Capitán" { "Capitán" }
                                }
                                label for="rango" { "Rango" }
                            }

                            div class="field label border" {
                                select id="estado" name="estado" required {
                                    option value="Activo" { "Activo" }
                                    option value="Licencia" { "Licencia" }
                                    option value="Retirado" { "Retirado" }
                                }
                                label for="estado" { "Estado en Servicio" }
                            }

                            div class="space" {}

                            button type="submit" class="primary round responsive" {
                                i { "add" }
                                span { "Dar de Alta" }
                            }
                        }
                    }
                }
                // Columna Derecha - Tabla (12/12 en móvil, 8/12 en tablet/escritorio)
                div class="s12 m8" {
                    article class="border round medium-padding" {
                        h5 class="medium-margin" {
                            "Personal Enlistado"
                        }
                        div class="responsive border" style="border-radius: 8px; overflow: hidden;" {
                            table class="stripes" {
                                thead {
                                    tr {
                                        th { "ID" }
                                        th { "Nombre" }
                                        th { "Rango" }
                                        th { "Estado" }
                                    }
                                }
                                tbody id="tabla-soldados" {
                                    (render_filas_soldados(&soldados))
                                }
                            }
                        }
                    }
                }
            }
            // Script para usar HTMX
            script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.10/dist/htmx.min.js" integrity="sha384-H5SrcfygHmAuTDZphMHqBJLc3FhssKjG7w/CeCpFReSfwBWDTKpkzPP8c+cLsK+V" crossorigin="anonymous" {}
        },
    )
}

// Handler POST: Incorpora y retorna solo las filas actualizadas
pub async fn agregar_soldado(
    State(state): State<AppState>,
    Form(nuevo): Form<NuevoSoldado>,
) -> Markup {
    if let Ok(conn) = state.db.connect() {
        let _ = conn
            .execute(
                "INSERT INTO soldados (nombre, rango, estado) VALUES (?1, ?2, ?3)",
                (nuevo.nombre, nuevo.rango, nuevo.estado),
            )
            .await;
    }

    let soldados = obtener_soldados(&state).await.unwrap_or_default();
    render_filas_soldados(&soldados)
}
