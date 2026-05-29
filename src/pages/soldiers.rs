use crate::AppState;
use crate::domain::types::{EstadoServicio, Rango};
use crate::layouts::main_layout::{PageContext, layout}; // Importamos nuestro Layout
use crate::security::CsrfToken;
use axum::{Extension, Form, extract::State};
use maud::{Markup, html};
use serde::Deserialize;
// Estructura para deserializar el formulario que envía HTMX
#[derive(Deserialize)]
pub struct NuevoSoldado {
    pub nombre: String,
    pub rango: Rango,
    pub estado: EstadoServicio,
}

// Estructura interna de datos
pub struct Soldado {
    pub id: i64,
    pub nombre: String,
    pub rango: Rango,
    pub estado: EstadoServicio,
}

// Función auxiliar para obtener soldados de la BD
async fn obtener_soldados(state: &AppState) -> Result<Vec<Soldado>, String> {
    let conn = state.db.connect().map_err(|e| {
        tracing::error!(error = %e, "Fallo al conectar para obtener soldados");
        e.to_string()
    })?;
    let mut rows = conn
        .query("SELECT id, nombre, rango, estado FROM soldados", ())
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Fallo en consulta SELECT de soldados");
            e.to_string()
        })?;
    let mut lista = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| {
        tracing::error!(error = %e, "Fallo al iterar filas de soldados");
        e.to_string()
    })? {
        let rango_str: String = row.get(2).map_err(|e| e.to_string())?;
        let estado_str: String = row.get(3).map_err(|e| e.to_string())?;

        let rango = rango_str.parse::<Rango>().map_err(|e| e.to_string())?;
        let estado = estado_str
            .parse::<EstadoServicio>()
            .map_err(|e| e.to_string())?;

        lista.push(Soldado {
            id: row.get(0).map_err(|e| e.to_string())?,
            nombre: row.get(1).map_err(|e| e.to_string())?,
            rango,
            estado,
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
                        @if s.estado == EstadoServicio::Activo { "primary-container" }
                        @else if s.estado == EstadoServicio::Licencia { "tertiary-container" }
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
pub async fn pagina_soldiers(
    State(state): State<AppState>,
    Extension(csrf_token): Extension<CsrfToken>,
) -> Markup {
    use tracing::Instrument;

    let span = tracing::info_span!("pagina_soldiers");
    async {
        tracing::info!("Cargando panel de administración de soldados");
        let ctx = PageContext::new(
            "Armadillos - Personal Militar",
            "Administración de Soldados",
            "Armadillos - Página para añadición y visualización de soldados dentro del sistema",
            &csrf_token.0,
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
            },
        )
    }
    .instrument(span)
    .await
}

// Handler POST: Incorpora y retorna solo las filas actualizadas
pub async fn agregar_soldado(
    State(state): State<AppState>,
    Form(nuevo): Form<NuevoSoldado>,
) -> Markup {
    use tracing::Instrument;

    let span = tracing::info_span!(
        "agregar_soldado",
        nombre = %nuevo.nombre,
        rango = ?nuevo.rango,
        estado = ?nuevo.estado
    );

    let db_result = async {
        tracing::info!("Iniciando intento de enlistamiento de nuevo soldado");
        match state.db.connect() {
            Ok(conn) => {
                match conn
                    .execute(
                        "INSERT INTO soldados (nombre, rango, estado) VALUES (?1, ?2, ?3)",
                        (
                            nuevo.nombre,
                            nuevo.rango.to_string(),
                            nuevo.estado.to_string(),
                        ),
                    )
                    .await
                {
                    Ok(_) => {
                        tracing::info!("Enlistamiento completado y persistido con éxito");
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Fallo al ejecutar inserción SQL");
                        Err(e.to_string())
                    }
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Fallo al conectar con la base de datos");
                Err(e.to_string())
            }
        }
    }
    .instrument(span)
    .await;

    if let Err(e) = db_result {
        tracing::warn!(
            "Procediendo con fallback de visualización tras error en persistencia: {}",
            e
        );
    }

    let soldados = obtener_soldados(&state).await.unwrap_or_default();
    render_filas_soldados(&soldados)
}
