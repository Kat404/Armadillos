use crate::AppState;
use crate::domain::EstadoServicio;
use crate::layouts::main_layout::{PageContext, layout};
use crate::security::CsrfToken;
use axum::{Extension, Form, extract::State};
use maud::{Markup, html};
use serde::Deserialize;

/// Formulario que envía HTMX al incorporar un nuevo soldado.
///
/// **¿Por qué `rango_id` y `seccion_servicio_id` en lugar de strings?**
/// Porque el nuevo esquema normalizado en 3FN almacena claves foráneas
/// enteras que referencian las tablas de lookup `rangos` y `secciones_servicios`.
/// Esto elimina el antipatrón "Stringly-Typed" a nivel de persistencia:
/// un ID inválido será rechazado por la restricción FOREIGN KEY de SQLite.
#[derive(Deserialize)]
pub struct NuevoSoldado {
    pub matricula: String,
    pub nombre: String,
    pub apellido_paterno: String,
    pub apellido_materno: Option<String>,
    pub rango_id: i64,
    pub seccion_servicio_id: i64,
    pub estado: EstadoServicio,
}

/// Representación interna de un soldado con sus datos resueltos de las
/// tablas de lookup (rango y sección como nombres legibles, no solo IDs).
pub struct Soldado {
    pub id: i64,
    pub matricula: String,
    pub nombre: String,
    pub apellido_paterno: String,
    pub apellido_materno: Option<String>,
    pub rango_nombre: String,
    pub seccion_nombre: String,
    pub estado: EstadoServicio,
}

/// Representación mínima de un rango para poblar el `<select>` del formulario.
pub struct RangoOption {
    pub id: i64,
    pub nombre: String,
}

/// Representación mínima de una sección de servicio para poblar el `<select>`.
pub struct SeccionOption {
    pub id: i64,
    pub nombre: String,
}

/// Obtiene la lista de rangos desde la BD para poblar el formulario.
async fn obtener_rangos(state: &AppState) -> Result<Vec<RangoOption>, String> {
    let conn = state.db.connect().map_err(|e| {
        tracing::error!(error = %e, "Fallo al conectar para obtener rangos");
        e.to_string()
    })?;
    let mut rows = conn
        .query(
            "SELECT id, nombre FROM rangos ORDER BY orden_jerarquico ASC",
            (),
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Fallo en consulta SELECT de rangos");
            e.to_string()
        })?;
    let mut lista = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        lista.push(RangoOption {
            id: row.get(0).map_err(|e| e.to_string())?,
            nombre: row.get(1).map_err(|e| e.to_string())?,
        });
    }
    Ok(lista)
}

/// Obtiene la lista de secciones de servicio desde la BD para poblar el formulario.
async fn obtener_secciones(state: &AppState) -> Result<Vec<SeccionOption>, String> {
    let conn = state.db.connect().map_err(|e| {
        tracing::error!(error = %e, "Fallo al conectar para obtener secciones");
        e.to_string()
    })?;
    let mut rows = conn
        .query(
            "SELECT id, nombre FROM secciones_servicios ORDER BY nombre ASC",
            (),
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Fallo en consulta SELECT de secciones");
            e.to_string()
        })?;
    let mut lista = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        lista.push(SeccionOption {
            id: row.get(0).map_err(|e| e.to_string())?,
            nombre: row.get(1).map_err(|e| e.to_string())?,
        });
    }
    Ok(lista)
}

/// Consulta los soldados con JOINs a las tablas de lookup para resolver
/// nombres legibles de rango y sección en lugar de IDs numéricos.
async fn obtener_soldados(state: &AppState) -> Result<Vec<Soldado>, String> {
    let conn = state.db.connect().map_err(|e| {
        tracing::error!(error = %e, "Fallo al conectar para obtener soldados");
        e.to_string()
    })?;
    let mut rows = conn
        .query(
            "SELECT s.id, s.matricula, s.nombre, s.apellido_paterno, s.apellido_materno,
                    r.nombre AS rango_nombre, sec.nombre AS seccion_nombre, s.estado
             FROM soldados s
             INNER JOIN rangos r ON s.rango_id = r.id
             INNER JOIN secciones_servicios sec ON s.seccion_servicio_id = sec.id
             ORDER BY s.id ASC",
            (),
        )
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
        let estado_str: String = row.get(7).map_err(|e| e.to_string())?;
        let estado = estado_str
            .parse::<EstadoServicio>()
            .map_err(|e| e.to_string())?;

        // Obtener apellido_materno que puede ser NULL en la BD
        let apellido_materno: Option<String> = row.get(4).ok();

        lista.push(Soldado {
            id: row.get(0).map_err(|e| e.to_string())?,
            matricula: row.get(1).map_err(|e| e.to_string())?,
            nombre: row.get(2).map_err(|e| e.to_string())?,
            apellido_paterno: row.get(3).map_err(|e| e.to_string())?,
            apellido_materno,
            rango_nombre: row.get(5).map_err(|e| e.to_string())?,
            seccion_nombre: row.get(6).map_err(|e| e.to_string())?,
            estado,
        });
    }
    Ok(lista)
}

/// Componente para renderizar las filas de la tabla de soldados.
fn render_filas_soldados(soldados: &[Soldado]) -> Markup {
    html! {
        @for s in soldados {
            tr {
                td { (s.id) }
                td { (s.matricula) }
                td {
                    (s.nombre) " " (s.apellido_paterno)
                    @if let Some(ref am) = s.apellido_materno {
                        " " (am)
                    }
                }
                td { (s.rango_nombre) }
                td { (s.seccion_nombre) }
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

/// Handler GET: Renderiza la página completa de gestión de soldados.
pub async fn pagina_soldados(
    State(state): State<AppState>,
    Extension(csrf_token): Extension<CsrfToken>,
) -> Markup {
    use tracing::Instrument;

    let span = tracing::info_span!("pagina_soldados");
    async {
        tracing::info!("Cargando panel de administración de soldados");
        let ctx = PageContext::new(
            "Armadillos - Personal Militar",
            "Administración de Soldados",
            "Armadillos - Página para añadición y visualización de soldados dentro del sistema",
            &csrf_token.0,
        );
        let soldados = obtener_soldados(&state).await.unwrap_or_default();
        let rangos = obtener_rangos(&state).await.unwrap_or_default();
        let secciones = obtener_secciones(&state).await.unwrap_or_default();

        layout(
            &ctx,
            html! {
                div class="grid" {
                    // Columna Izquierda - Formulario (12/12 en móvil, 4/12 en tablet/escritorio)
                    div class="s12 m4" {
                        article class="border round medium-padding" {
                            h5 class="medium-margin" {
                                "Incorporar Soldado"
                            }
                            form hx-post="/soldados" hx-target="#tabla-soldados" hx-swap="innerHTML" {
                                div class="field label border" {
                                    input type="text" id="matricula" name="matricula" placeholder=" " required;
                                    label for="matricula" { "Matrícula" }
                                }

                                div class="field label border" {
                                    input type="text" id="nombre" name="nombre" placeholder=" " required;
                                    label for="nombre" { "Nombre(s)" }
                                }

                                div class="field label border" {
                                    input type="text" id="apellido_paterno" name="apellido_paterno" placeholder=" " required;
                                    label for="apellido_paterno" { "Apellido Paterno" }
                                }

                                div class="field label border" {
                                    input type="text" id="apellido_materno" name="apellido_materno" placeholder=" ";
                                    label for="apellido_materno" { "Apellido Materno" }
                                }

                                div class="field label border" {
                                    select id="rango_id" name="rango_id" required {
                                        @for r in &rangos {
                                            option value=(r.id) { (r.nombre) }
                                        }
                                    }
                                    label for="rango_id" { "Rango" }
                                }

                                div class="field label border" {
                                    select id="seccion_servicio_id" name="seccion_servicio_id" required {
                                        @for sec in &secciones {
                                            option value=(sec.id) { (sec.nombre) }
                                        }
                                    }
                                    label for="seccion_servicio_id" { "Sección de Servicio" }
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
                                            th { "Matrícula" }
                                            th { "Nombre Completo" }
                                            th { "Rango" }
                                            th { "Sección" }
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

/// Handler POST: Incorpora un soldado y retorna las filas actualizadas.
pub async fn agregar_soldado(
    State(state): State<AppState>,
    Form(nuevo): Form<NuevoSoldado>,
) -> Markup {
    use tracing::Instrument;

    let span = tracing::info_span!(
        "agregar_soldado",
        matricula = %nuevo.matricula,
        rango_id = %nuevo.rango_id,
        estado = ?nuevo.estado
    );

    let db_result = async {
        tracing::info!("Iniciando intento de enlistamiento de nuevo soldado");
        match state.db.connect() {
            Ok(conn) => {
                // Utilizamos el apellido_materno como Option — si viene vacío del
                // formulario HTML, lo tratamos como NULL en la BD.
                let apellido_materno = nuevo
                    .apellido_materno
                    .as_deref()
                    .filter(|s| !s.trim().is_empty());

                match conn
                    .execute(
                        "INSERT INTO soldados (matricula, nombre, apellido_paterno, apellido_materno, rango_id, seccion_servicio_id, estado)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        (
                            nuevo.matricula,
                            nuevo.nombre,
                            nuevo.apellido_paterno,
                            apellido_materno.map(|s| s.to_string()),
                            nuevo.rango_id,
                            nuevo.seccion_servicio_id,
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

/// Formulario para simular el operador actual en sesión.
#[derive(Deserialize)]
pub struct SimularOperador {
    pub operador_id: i64,
    pub redir_path: Option<String>,
}

/// Handler POST `/simular_operador`: Establece la cookie de simulación del operador actual.
pub async fn simular_operador(
    Form(payload): Form<SimularOperador>,
) -> impl axum::response::IntoResponse {
    use axum::http::{HeaderMap, header};
    use axum::response::Redirect;

    let mut headers = HeaderMap::new();
    let cookie = format!(
        "operador_soldado_id={}; Path=/; SameSite=Strict; HttpOnly",
        payload.operador_id
    );
    headers.insert(
        header::SET_COOKIE,
        axum::http::HeaderValue::from_str(&cookie).unwrap(),
    );

    let redirect_url = payload
        .redir_path
        .unwrap_or_else(|| "/soldados".to_string());
    (headers, Redirect::to(&redirect_url))
}

/// Estructura que consolida los datos legibles y el contexto de seguridad del operador simulado.
pub struct DatosOperador {
    pub id: i64,
    pub nombre_completo: String,
    pub rango_nombre: String,
    pub seccion_nombre: String,
    pub contexto: crate::domain::ContextoAcceso,
}

/// Resuelve el operador militar activo leyendo la cookie e interrogando la base de datos.
/// Si no hay cookie, utiliza el primer soldado registrado en el sistema como fallback por defecto.
pub async fn resolver_operador_activo(
    headers: &axum::http::HeaderMap,
    state: &crate::AppState,
) -> Result<Option<DatosOperador>, String> {
    use crate::domain::{ContextoAcceso, Rango};
    use crate::security::extraer_cookie;

    let operador_id_opt = extraer_cookie(headers, "operador_soldado_id")
        .and_then(|id_str| id_str.parse::<i64>().ok());

    let conn = state.db.connect().map_err(|e| e.to_string())?;

    let operador_id = match operador_id_opt {
        Some(id) => id,
        None => {
            let mut rows = conn
                .query("SELECT id FROM soldados ORDER BY id ASC LIMIT 1", ())
                .await
                .map_err(|e| e.to_string())?;
            if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
                row.get(0).map_err(|e| e.to_string())?
            } else {
                return Ok(None);
            }
        }
    };

    let mut rows = conn
        .query(
            "SELECT s.id, s.nombre, s.apellido_paterno, s.rango_id, sec.nombre, sec.es_servicio_belico, r.nombre
             FROM soldados s
             JOIN secciones_servicios sec ON s.seccion_servicio_id = sec.id
             JOIN rangos r ON s.rango_id = r.id
             WHERE s.id = ?1",
            (operador_id,),
        )
        .await
        .map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let nombre: String = row.get(1).map_err(|e| e.to_string())?;
        let apellido: String = row.get(2).map_err(|e| e.to_string())?;
        let rango_id: i64 = row.get(3).map_err(|e| e.to_string())?;
        let seccion_nombre: String = row.get(4).map_err(|e| e.to_string())?;
        let es_servicio_belico_num: i64 = row.get(5).map_err(|e| e.to_string())?;
        let rango_nombre: String = row.get(6).map_err(|e| e.to_string())?;

        let es_servicio_belico = es_servicio_belico_num != 0;
        let rango = Rango::from_id(rango_id)?;

        Ok(Some(DatosOperador {
            id,
            nombre_completo: format!("{} {}", nombre, apellido),
            rango_nombre,
            seccion_nombre,
            contexto: ContextoAcceso {
                rango,
                es_servicio_belico,
            },
        }))
    } else {
        Ok(None)
    }
}
