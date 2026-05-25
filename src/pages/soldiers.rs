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
                    span class = {
                        "badge "
                        @if s.estado == "Activo" { "success" }
                        @else if s.estado == "Licencia" { "warning" }
                        @else { "secondary" }
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
            h2 {
                "Panel de Control de Soldados"
            }
            div class="grid" {
                // Columna Izquierda
                div {
                    h3 {
                        "Incorporar Soldados"
                    }
                    form hx-post="/soldiers" hx-target="#tabla-soldados" hx-swap="innerHTML" {
                        label for="nombre" {
                            "Nombre Completo"
                            input type="text" id="nombre" name="nombre" placeholder="Ej: Juan Carlos Rodríguez" required;
                        }

                        label for="rango" {
                            "Rango"
                            select id="rango" name="rango" required {
                                option value="Soldado" { "Soldado" }
                                option value="Cabo" { "Cabo" }
                                option value="Sargento" { "Sargento" }
                                option value="Teniente" { "Teniente" }
                                option value="Capitán" { "Capitán" }
                            }
                        }

                        label for="estado" {
                            "Estado en Servicio"
                            select id="estado" name="estado" required {
                                option value="Activo" { "Activo" }
                                option value="Licencia" { "Licencia" }
                                option value="Retirado" { "Retirado" }
                            }
                        }

                        button type="submit" {
                            "Dar de Alta"
                        }
                    }
                }
                // Columna Derecha
                div {
                    h3 {
                        "Personal Enlistado"
                    }
                    table {
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
