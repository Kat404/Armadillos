#![allow(dead_code)]
use crate::AppState;
use crate::layouts::main_layout::{PageContext, layout};
use crate::pages::inventario::SoldadoOpcion;
use crate::pages::soldados::resolver_operador_activo;
use crate::security::CsrfToken;
use axum::{
    Extension, Form,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use maud::{Markup, html};
use serde::Deserialize;

/// Genera un timestamp UTC en formato ISO 8601 compatible con SQLite.
/// Utiliza la syscall del sistema en lugar de un crate externo.
fn chrono_now_utc() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Error al obtener la hora del sistema");
    let secs = now.as_secs();
    // Cálculo de fecha/hora sin dependencia externa
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Algoritmo civil de days desde epoch (2000-03-01 base)
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        y, m, d, hours, minutes, seconds
    )
}

/// Representación para el historial de cadena de custodia (modelo ledger append-only).
pub struct DetalleAsignacion {
    pub id: i64,
    pub fecha_asignacion: String,
    pub cantidad: i64,
    pub receptor_nombre: String,
    pub receptor_rango: String,
    pub equipo_nombre: String,
    pub equipo_codigo: String,
    pub categoria_nombre: String,
    pub es_belico: bool,
    pub autorizante_nombre: String,
    pub autorizante_rango: String,
    /// Indica si este evento de asignación ya fue devuelto (existe un evento DEVOLUCION asociado).
    pub esta_devuelto: bool,
}

/// Representación del equipamiento disponible para asignar.
pub struct EquipamientoOpcion {
    pub id: i64,
    pub nombre: String,
    pub codigo_inventario: String,
    pub stock_disponible: i64,
    pub categoria_nombre: String,
    pub es_material_de_guerra: bool,
}

/// Obtiene todos los soldados disponibles para poblar el selector de receptores.
async fn obtener_soldados_opciones(state: &AppState) -> Result<Vec<SoldadoOpcion>, String> {
    let conn = state.db.connect().map_err(|e| e.to_string())?;
    let mut rows = conn
        .query(
            "SELECT s.id, s.nombre_encriptado, s.apellido_paterno_encriptado, r.nombre
             FROM soldados s
             JOIN rangos r ON s.rango_id = r.id
             ORDER BY r.orden_jerarquico DESC",
            (),
        )
        .await
        .map_err(|e| e.to_string())?;
    let mut lista = Vec::new();
    let ale_key = crate::crypto::obtener_ale_key();
    use secrecy::ExposeSecret;

    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let nombre_enc: String = row.get(1).map_err(|e| e.to_string())?;
        let apellido_enc: String = row.get(2).map_err(|e| e.to_string())?;

        let nombre = crate::crypto::desencriptar_pii(&nombre_enc, &ale_key)
            .map_err(|e| format!("Error descifrando nombre opcion: {}", e))?
            .expose_secret()
            .to_string();
        let apellido = crate::crypto::desencriptar_pii(&apellido_enc, &ale_key)
            .map_err(|e| format!("Error descifrando apellido opcion: {}", e))?
            .expose_secret()
            .to_string();

        lista.push(SoldadoOpcion {
            id: row.get(0).map_err(|e| e.to_string())?,
            nombre,
            apellido,
            rango_nombre: row.get(3).map_err(|e| e.to_string())?,
        });
    }
    Ok(lista)
}

/// Obtiene el equipamiento operativo que tiene stock disponible.
async fn obtener_equipamiento_opciones(
    state: &AppState,
) -> Result<Vec<EquipamientoOpcion>, String> {
    let conn = state.db.connect().map_err(|e| e.to_string())?;
    let mut rows = conn
        .query(
            "SELECT e.id, e.nombre, e.codigo_inventario, e.stock_disponible, c.nombre, c.es_material_de_guerra
             FROM equipamiento e
             JOIN categorias_equipamiento c ON e.categoria_id = c.id
             WHERE e.stock_disponible > 0 AND e.estado_conservacion = 'Operativo'
             ORDER BY c.nombre ASC, e.nombre ASC",
            (),
        )
        .await
        .map_err(|e| e.to_string())?;
    let mut lista = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let es_material_de_guerra_num: i64 = row.get(5).map_err(|e| e.to_string())?;
        lista.push(EquipamientoOpcion {
            id: row.get(0).map_err(|e| e.to_string())?,
            nombre: row.get(1).map_err(|e| e.to_string())?,
            codigo_inventario: row.get(2).map_err(|e| e.to_string())?,
            stock_disponible: row.get(3).map_err(|e| e.to_string())?,
            categoria_nombre: row.get(4).map_err(|e| e.to_string())?,
            es_material_de_guerra: es_material_de_guerra_num != 0,
        });
    }
    Ok(lista)
}

/// Obtiene el listado de asignaciones activas (tipo_evento = 'ASIGNACION')
/// con detección relacional de devoluciones mediante LEFT JOIN.
async fn obtener_asignaciones(state: &AppState) -> Result<Vec<DetalleAsignacion>, String> {
    let conn = state.db.connect().map_err(|e| e.to_string())?;
    let mut rows = conn
        .query(
            "SELECT a.id, a.fecha_evento, a.cantidad,
                    s_rec.nombre_encriptado AS receptor_nombre_enc,
                    s_rec.apellido_paterno_encriptado AS receptor_apellido_enc,
                    r_rec.nombre AS receptor_rango,
                    e.nombre AS equipo_nombre,
                    e.codigo_inventario AS equipo_codigo,
                    c.nombre AS categoria_nombre,
                    c.es_material_de_guerra AS es_belico,
                    s_aut.nombre_encriptado AS autorizante_nombre_enc,
                    s_aut.apellido_paterno_encriptado AS autorizante_apellido_enc,
                    r_aut.nombre AS autorizante_rango,
                    (d.id IS NOT NULL) AS esta_devuelto
             FROM asignaciones_equipamiento a
             JOIN soldados s_rec ON a.soldado_id = s_rec.id
             JOIN rangos r_rec ON s_rec.rango_id = r_rec.id
             JOIN equipamiento e ON a.equipamiento_id = e.id
             JOIN categorias_equipamiento c ON e.categoria_id = c.id
             JOIN soldados s_aut ON a.autorizado_por_soldado_id = s_aut.id
             JOIN rangos r_aut ON s_aut.rango_id = r_aut.id
             LEFT JOIN asignaciones_equipamiento d ON d.tipo_evento = 'DEVOLUCION' AND d.asignacion_origen_id = a.id
             WHERE a.tipo_evento = 'ASIGNACION'
             ORDER BY a.fecha_evento DESC",
            (),
        )
        .await
        .map_err(|e| e.to_string())?;

    let mut lista = Vec::new();
    let ale_key = crate::crypto::obtener_ale_key();
    use secrecy::ExposeSecret;

    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let es_belico_num: i64 = row.get(9).map_err(|e| e.to_string())?;
        let devuelto_num: i64 = row.get(13).map_err(|e| e.to_string())?;

        let rec_nombre_enc: String = row.get(3).map_err(|e| e.to_string())?;
        let rec_apellido_enc: String = row.get(4).map_err(|e| e.to_string())?;
        let aut_nombre_enc: String = row.get(10).map_err(|e| e.to_string())?;
        let aut_apellido_enc: String = row.get(11).map_err(|e| e.to_string())?;

        let rec_nombre = crate::crypto::desencriptar_pii(&rec_nombre_enc, &ale_key)
            .map_err(|e| format!("Error descifrando receptor nombre: {}", e))?
            .expose_secret()
            .to_string();
        let rec_apellido = crate::crypto::desencriptar_pii(&rec_apellido_enc, &ale_key)
            .map_err(|e| format!("Error descifrando receptor apellido: {}", e))?
            .expose_secret()
            .to_string();

        let aut_nombre = crate::crypto::desencriptar_pii(&aut_nombre_enc, &ale_key)
            .map_err(|e| format!("Error descifrando autorizante nombre: {}", e))?
            .expose_secret()
            .to_string();
        let aut_apellido = crate::crypto::desencriptar_pii(&aut_apellido_enc, &ale_key)
            .map_err(|e| format!("Error descifrando autorizante apellido: {}", e))?
            .expose_secret()
            .to_string();

        lista.push(DetalleAsignacion {
            id: row.get(0).map_err(|e| e.to_string())?,
            fecha_asignacion: row.get(1).map_err(|e| e.to_string())?,
            cantidad: row.get(2).map_err(|e| e.to_string())?,
            receptor_nombre: format!("{} {}", rec_nombre, rec_apellido),
            receptor_rango: row.get(5).map_err(|e| e.to_string())?,
            equipo_nombre: row.get(6).map_err(|e| e.to_string())?,
            equipo_codigo: row.get(7).map_err(|e| e.to_string())?,
            categoria_nombre: row.get(8).map_err(|e| e.to_string())?,
            es_belico: es_belico_num != 0,
            autorizante_nombre: format!("{} {}", aut_nombre, aut_apellido),
            autorizante_rango: row.get(12).map_err(|e| e.to_string())?,
            esta_devuelto: devuelto_num != 0,
        });
    }
    Ok(lista)
}

/// Renders the inner content of the assignments page, which can be swapped asynchronously by HTMX.
async fn render_fragmento_asignaciones(
    state: &AppState,
    jar: &axum_extra::extract::PrivateCookieJar,
    csrf_token: &str,
    alerta_markup: Option<Markup>,
) -> Markup {
    let operador = resolver_operador_activo(jar, state).await.unwrap_or(None);
    let todos_soldados = obtener_soldados_opciones(state).await.unwrap_or_default();
    let equipamiento = obtener_equipamiento_opciones(state)
        .await
        .unwrap_or_default();
    let asignaciones = obtener_asignaciones(state).await.unwrap_or_default();

    let op_nombre = operador.as_ref().map(|o| o.nombre_completo.clone());
    let op_rango = operador.as_ref().map(|o| o.rango_nombre.clone());
    let op_seccion = operador.as_ref().map(|o| o.seccion_nombre.clone());
    let puede_belico = operador
        .as_ref()
        .map(|o| o.contexto.puede_administrar_inventario_belico())
        .unwrap_or(false);

    html! {
        div id="seccion-asignaciones-completo" {
            // 1. Info del Operador en sesión (Componentizado)
            (crate::components::operador::info_operador(
                op_nombre.as_deref(),
                op_rango.as_deref(),
                op_seccion.as_deref(),
                puede_belico,
            ))

            // 2. Banner de alertas inyectado dinámicamente si existe
            @if let Some(alerta) = alerta_markup {
                (alerta)
            }

            div class="grid" {
                // Columna Izquierda: Formulario de Asignación
                div class="s12 m4" {
                    article class="border round medium-padding" {
                        h5 class="medium-margin" { "Asignar Equipo / Armamento" }

                        form hx-post="/asignaciones" hx-target="#seccion-asignaciones-completo" hx-swap="outerHTML" {
                            input type="hidden" name="csrf_token" value=(csrf_token);

                            div class="field label border" {
                                select id="soldado_id" name="soldado_id" required {
                                    @for s in &todos_soldados {
                                        option value=(s.id) { (s.rango_nombre) " — " (s.nombre) " " (s.apellido) }
                                    }
                                }
                                label for="soldado_id" { "Personal Receptor" }
                            }

                            div class="field label border" {
                                select id="equipamiento_id" name="equipamiento_id" required {
                                    @for e in &equipamiento {
                                        @let no_permitido = e.es_material_de_guerra && !puede_belico;
                                        option value=(e.id) disabled?[no_permitido] {
                                            (e.nombre) " (" (e.codigo_inventario) ") [Disp: " (e.stock_disponible) "]"
                                            @if e.es_material_de_guerra {
                                                @if no_permitido { " (⚠ Bélico — Bloqueado por ABAC)" } @else { " (Bélico ⚠)" }
                                            }
                                        }
                                    }
                                }
                                label for="equipamiento_id" { "Equipamiento / Ítem" }
                            }

                            div class="field label border" {
                                input type="number" id="cantidad" name="cantidad" min="1" value="1" placeholder=" " required;
                                label for="cantidad" { "Cantidad" }
                            }

                            div class="medium-margin padding border round surface-variant" {
                                p class="caption no-margin text-secondary" { "Autorizado por (Usuario simulado en sesión):" }
                                @if let Some(ref op) = operador {
                                    p class="bold no-margin text-primary" { (op.rango_nombre) " — " (op.nombre_completo) }
                                } @else {
                                    p class="bold no-margin text-error" { "Ninguno (Seleccione un operador arriba)" }
                                }
                            }

                            button type="submit" class="responsive primary" {
                                i { "assignment_turned_in" }
                                span { "Completar Asignación" }
                            }
                        }
                    }
                }

                // Columna Derecha: Bitácora de cadena de custodia
                div class="s12 m8" {
                    article class="border round medium-padding" {
                        div class="row align-center space-between medium-margin" {
                            h5 class="no-margin" { "Registro Histórico de Entregas" }
                            button class="responsive surface-variant"
                                hx-get="/asignaciones/auditoria"
                                hx-target="#resultado-auditoria"
                                hx-swap="innerHTML" {
                                i { "security" }
                                span { "Auditar Integridad (BLAKE3)" }
                            }
                        }
                        div id="resultado-auditoria" class="medium-margin" {}

                        div class="table-container" {
                            table class="striped hover" {
                                thead {
                                    tr {
                                        th { "Fecha" }
                                        th { "Equipo" }
                                        th { "Cant." }
                                        th { "Receptor" }
                                        th { "Autorizado Por" }
                                        th class="center-align" { "Acción" }
                                    }
                                }
                                tbody {
                                    @if asignaciones.is_empty() {
                                        tr {
                                            td colspan="6" class="center-align text-secondary" { "No se han realizado asignaciones en la bitácora." }
                                        }
                                    }
                                    @for asig in &asignaciones {
                                        tr {
                                            td class="caption font-mono" { (asig.fecha_asignacion) }
                                            td {
                                                div class="bold" {
                                                    (asig.equipo_nombre) " (" (asig.equipo_codigo) ")"
                                                    @if asig.es_belico {
                                                        span class="badge none red circle" style="margin-left: 8px" { "⚠ Bélico" }
                                                    }
                                                }
                                                div class="caption text-secondary" { (asig.categoria_nombre) }
                                            }
                                            td class="center-align bold" { (asig.cantidad) }
                                            td {
                                                div class="bold" { (asig.receptor_nombre) }
                                                div class="caption text-secondary" { (asig.receptor_rango) }
                                            }
                                            td {
                                                div class="bold text-primary" { (asig.autorizante_nombre) }
                                                div class="caption text-secondary" { (asig.autorizante_rango) }
                                            }
                                            td class="center-align" {
                                                @if !asig.esta_devuelto {
                                                    form hx-post="/asignaciones/devolver" hx-target="#seccion-asignaciones-completo" hx-swap="outerHTML" class="no-margin" {
                                                        input type="hidden" name="csrf_token" value=(csrf_token);
                                                        input type="hidden" name="asignacion_id" value=(asig.id);
                                                        button type="submit" class="circle small error transparent no-margin" title="Registrar Devolución de Equipo" {
                                                            i { "keyboard_return" }
                                                        }
                                                    }
                                                } @else {
                                                    span class="text-secondary italic caption" { "Devuelto" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Handler GET `/asignaciones`: Muestra la página de asignaciones.
pub async fn pagina_asignaciones(
    State(state): State<AppState>,
    Extension(csrf_token): Extension<CsrfToken>,
    jar: axum_extra::extract::PrivateCookieJar,
) -> impl axum::response::IntoResponse {
    use axum::response::Redirect;
    if jar.get("operador_soldado_id").is_none() {
        return Redirect::to("/login").into_response();
    }

    let ctx = PageContext::new(
        "Armadillos - Asignaciones",
        "Cadena de Custodia de Equipamiento",
        "Registro y control del armamento y equipo asignado individualmente al personal militar.",
        &csrf_token.0,
    );

    let fragmento = render_fragmento_asignaciones(&state, &jar, &csrf_token.0, None).await;

    layout(&ctx, fragmento).into_response()
}

/// Estructura que captura los datos de una nueva asignación de equipamiento.
#[derive(Deserialize)]
pub struct NuevaAsignacion {
    pub soldado_id: i64,
    pub equipamiento_id: i64,
    pub cantidad: i64,
}

/// Handler POST `/asignaciones`: Registra una asignación bajo transacción SQLite y validación ABAC.
pub async fn crear_asignacion(
    State(state): State<AppState>,
    Extension(csrf_token): Extension<CsrfToken>,
    jar: axum_extra::extract::PrivateCookieJar,
    Form(payload): Form<NuevaAsignacion>,
) -> Response {
    // 1. Resolver el Operador Activo
    let operador = match resolver_operador_activo(&jar, &state).await {
        Ok(Some(op)) => op,
        _ => {
            let error_html = crate::components::alerta::alerta(
                "error",
                "Sesión Inválida",
                "No hay una sesión de operador activa o válida.",
            );
            let fragmento =
                render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
            return (StatusCode::FORBIDDEN, fragmento).into_response();
        }
    };

    // 2. Conectar a la base de datos
    let conn = match state.db.connect() {
        Ok(c) => c,
        Err(e) => {
            let error_html =
                crate::components::alerta::alerta("error", "Fallo de Conexión", &e.to_string());
            let fragmento =
                render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
            return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
        }
    };

    // 3. Iniciar Transacción Manual para asegurar consistencia ACID
    if let Err(e) = conn.execute("BEGIN TRANSACTION", ()).await {
        let error_html =
            crate::components::alerta::alerta("error", "Error de Transacción", &e.to_string());
        let fragmento =
            render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
    }

    // 4. Consultar stock disponible y categoría del equipo
    let row_result = conn
        .query(
            "SELECT e.stock_disponible, c.es_material_de_guerra, e.nombre
             FROM equipamiento e
             JOIN categorias_equipamiento c ON e.categoria_id = c.id
             WHERE e.id = ?1",
            (payload.equipamiento_id,),
        )
        .await;

    let (stock_disponible, es_material_de_guerra, equipo_nombre) = match row_result {
        Ok(mut rows) => match rows.next().await {
            Ok(Some(row)) => {
                let stock: i64 = row.get(0).unwrap_or(0);
                let es_belico_num: i64 = row.get(1).unwrap_or(0);
                let nombre: String = row.get(2).unwrap_or_else(|_| "Equipo".to_string());
                (stock, es_belico_num != 0, nombre)
            }
            _ => {
                let _ = conn.execute("ROLLBACK", ()).await;
                let error_html = crate::components::alerta::alerta(
                    "error",
                    "Equipo No Encontrado",
                    "El equipamiento seleccionado no existe en inventario.",
                );
                let fragmento =
                    render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html))
                        .await;
                return (StatusCode::BAD_REQUEST, fragmento).into_response();
            }
        },
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ()).await;
            let error_html =
                crate::components::alerta::alerta("error", "Error de Consulta SQL", &e.to_string());
            let fragmento =
                render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
            return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
        }
    };

    // 5. Validar ABAC para material de guerra
    if es_material_de_guerra && !operador.contexto.puede_administrar_inventario_belico() {
        let _ = conn.execute("ROLLBACK", ()).await;
        tracing::warn!(
            operador_id = %operador.id,
            rango = %operador.rango_nombre,
            "Intento de asignación de material de guerra bloqueado por ABAC"
        );
        let error_msg = format!(
            "El autorizante simulado ({} — {}) carece de privilegios para asignar material de guerra sensible.",
            operador.rango_nombre, operador.nombre_completo
        );
        let error_html =
            crate::components::alerta::alerta("error", "PERMISO DENEGADO (ABAC)", &error_msg);
        let fragmento =
            render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::FORBIDDEN, fragmento).into_response();
    }

    // 6. Validar disponibilidad de stock
    if stock_disponible < payload.cantidad {
        let _ = conn.execute("ROLLBACK", ()).await;
        let error_msg = format!(
            "Intenta asignar {} unidades de '{}', pero solo quedan {} en inventario.",
            payload.cantidad, equipo_nombre, stock_disponible
        );
        let error_html =
            crate::components::alerta::alerta("error", "STOCK INSUFICIENTE", &error_msg);
        let fragmento =
            render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::BAD_REQUEST, fragmento).into_response();
    }

    // 7. Calcular el hash-chain BLAKE3 y registrar el evento de ASIGNACIÓN en el ledger
    let fecha_evento = chrono_now_utc();
    let hash_anterior = match crate::security::obtener_ultimo_hash(&conn).await {
        Ok(h) => h,
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ()).await;
            let error_html = crate::components::alerta::alerta("error", "Error de Hash-Chain", &e);
            let fragmento =
                render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
            return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
        }
    };

    let hash_verificacion = crate::security::calcular_hash_evento(
        "ASIGNACION",
        payload.soldado_id,
        payload.equipamiento_id,
        payload.cantidad,
        &fecha_evento,
        operador.id,
        None,
        &hash_anterior,
    );

    if let Err(e) = conn
        .execute(
            "INSERT INTO asignaciones_equipamiento (tipo_evento, soldado_id, equipamiento_id, cantidad, fecha_evento, autorizado_por_soldado_id, hash_verificacion)
             VALUES ('ASIGNACION', ?1, ?2, ?3, ?4, ?5, ?6)",
            (
                payload.soldado_id,
                payload.equipamiento_id,
                payload.cantidad,
                fecha_evento.clone(),
                operador.id,
                hash_verificacion,
            ),
        )
        .await
    {
        let _ = conn.execute("ROLLBACK", ()).await;
        let error_html = crate::components::alerta::alerta("error", "Error de Inserción SQL", &e.to_string());
        let fragmento = render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
    }

    // 8. Descontar stock disponible en la tabla equipamiento
    if let Err(e) = conn
        .execute(
            "UPDATE equipamiento SET stock_disponible = stock_disponible - ?1 WHERE id = ?2",
            (payload.cantidad, payload.equipamiento_id),
        )
        .await
    {
        let _ = conn.execute("ROLLBACK", ()).await;
        let error_html = crate::components::alerta::alerta(
            "error",
            "Error de Actualización de Stock",
            &e.to_string(),
        );
        let fragmento =
            render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
    }

    // 9. Completar la transacción
    if let Err(e) = conn.execute("COMMIT", ()).await {
        let _ = conn.execute("ROLLBACK", ()).await;
        let error_html =
            crate::components::alerta::alerta("error", "Error de Commit", &e.to_string());
        let fragmento =
            render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
    }

    tracing::info!(
        operador_id = %operador.id,
        equipo = %equipo_nombre,
        cantidad = %payload.cantidad,
        "Asignación de equipamiento autorizada y registrada con éxito"
    );

    // Éxito: retornamos la vista parcial de asignaciones limpia y un banner de éxito
    let exito_html = crate::components::alerta::alerta(
        "success",
        "Asignación Exitosa",
        "El equipamiento ha sido asignado y el stock se ha actualizado correctamente.",
    );
    let fragmento =
        render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(exito_html)).await;
    fragmento.into_response()
}

/// Estructura que captura los datos de devolución de equipamiento.
#[derive(Deserialize)]
pub struct DevolucionAsignacion {
    pub asignacion_id: i64,
}

/// Handler POST `/asignaciones/devolver`: Retorna el equipamiento asignado a la BD de forma segura bajo transacción ACID.
pub async fn devolver_asignacion(
    State(state): State<AppState>,
    Extension(csrf_token): Extension<CsrfToken>,
    jar: axum_extra::extract::PrivateCookieJar,
    Form(payload): Form<DevolucionAsignacion>,
) -> Response {
    // 1. Resolver el Operador Activo (quien procesa el retorno de armamento)
    let operador = match resolver_operador_activo(&jar, &state).await {
        Ok(Some(op)) => op,
        _ => {
            let error_html = crate::components::alerta::alerta(
                "error",
                "Sesión Inválida",
                "No hay una sesión de operador activa o válida.",
            );
            let fragmento =
                render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
            return (StatusCode::FORBIDDEN, fragmento).into_response();
        }
    };

    // 2. Conectar a la base de datos
    let conn = match state.db.connect() {
        Ok(c) => c,
        Err(e) => {
            let error_html =
                crate::components::alerta::alerta("error", "Fallo de Conexión", &e.to_string());
            let fragmento =
                render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
            return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
        }
    };

    // 3. Consultar datos de la asignación origen y verificar que no fue ya devuelta
    let mut rows = match conn
        .query(
            "SELECT a.equipamiento_id, a.cantidad, c.es_material_de_guerra, e.nombre, a.soldado_id
             FROM asignaciones_equipamiento a
             JOIN equipamiento e ON a.equipamiento_id = e.id
             JOIN categorias_equipamiento c ON e.categoria_id = c.id
             WHERE a.id = ?1 AND a.tipo_evento = 'ASIGNACION'",
            (payload.asignacion_id,),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            let error_html =
                crate::components::alerta::alerta("error", "Error de Consulta SQL", &e.to_string());
            let fragmento =
                render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
            return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
        }
    };

    let (equipamiento_id, cantidad, es_belico, equipo_nombre, soldado_id) = match rows.next().await
    {
        Ok(Some(row)) => {
            let eq_id: i64 = row.get(0).unwrap();
            let cant: i64 = row.get(1).unwrap();
            let belico_num: i64 = row.get(2).unwrap();
            let nombre: String = row.get(3).unwrap();
            let sol_id: i64 = row.get(4).unwrap();
            (eq_id, cant, belico_num != 0, nombre, sol_id)
        }
        _ => {
            let error_html = crate::components::alerta::alerta(
                "error",
                "Asignación Inexistente",
                "No se encontró el registro de asignación especificado.",
            );
            let fragmento =
                render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
            return (StatusCode::BAD_REQUEST, fragmento).into_response();
        }
    };

    // 3b. Verificar que no exista ya una devolución para esta asignación (idempotencia)
    let ya_devuelta = match conn
        .query(
            "SELECT id FROM asignaciones_equipamiento WHERE tipo_evento = 'DEVOLUCION' AND asignacion_origen_id = ?1 LIMIT 1",
            (payload.asignacion_id,),
        )
        .await
    {
        Ok(mut dev_rows) => dev_rows.next().await.ok().flatten().is_some(),
        Err(_) => false,
    };

    if ya_devuelta {
        let error_html = crate::components::alerta::alerta(
            "error",
            "Devolución Duplicada",
            "Esta asignación ya fue devuelta anteriormente.",
        );
        let fragmento =
            render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::BAD_REQUEST, fragmento).into_response();
    }

    // 4. Evaluar la política ABAC si el equipo a devolver es Material de Guerra
    if es_belico && !operador.contexto.puede_administrar_inventario_belico() {
        tracing::warn!(
            operador_id = %operador.id,
            rango = %operador.rango_nombre,
            "Intento de devolución de material de guerra bloqueado por ABAC"
        );
        let error_msg = format!(
            "El operador simulado ({} — {}) no tiene autorización para registrar la devolución de material de guerra.",
            operador.rango_nombre, operador.nombre_completo
        );
        let error_html =
            crate::components::alerta::alerta("error", "PERMISO DENEGADO (ABAC)", &error_msg);
        let fragmento =
            render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::FORBIDDEN, fragmento).into_response();
    }

    // 5. Iniciar la transacción SQLite
    if let Err(e) = conn.execute("BEGIN TRANSACTION", ()).await {
        let error_html =
            crate::components::alerta::alerta("error", "Error de Transacción", &e.to_string());
        let fragmento =
            render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
    }

    // 6. Calcular hash-chain e insertar evento de DEVOLUCIÓN (append-only, nunca UPDATE)
    let fecha_evento = chrono_now_utc();
    let hash_anterior = match crate::security::obtener_ultimo_hash(&conn).await {
        Ok(h) => h,
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ()).await;
            let error_html = crate::components::alerta::alerta("error", "Error de Hash-Chain", &e);
            let fragmento =
                render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
            return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
        }
    };

    let hash_verificacion = crate::security::calcular_hash_evento(
        "DEVOLUCION",
        soldado_id,
        equipamiento_id,
        cantidad,
        &fecha_evento,
        operador.id,
        Some(payload.asignacion_id),
        &hash_anterior,
    );

    if let Err(e) = conn
        .execute(
            "INSERT INTO asignaciones_equipamiento (tipo_evento, soldado_id, equipamiento_id, cantidad, fecha_evento, autorizado_por_soldado_id, asignacion_origen_id, hash_verificacion)
             VALUES ('DEVOLUCION', ?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                soldado_id,
                equipamiento_id,
                cantidad,
                fecha_evento.clone(),
                operador.id,
                payload.asignacion_id,
                hash_verificacion,
            ),
        )
        .await
    {
        let _ = conn.execute("ROLLBACK", ()).await;
        let error_html = crate::components::alerta::alerta(
            "error",
            "Error de Inserción de Devolución",
            &e.to_string(),
        );
        let fragmento =
            render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
    }

    // 7. Devolver el stock disponible en la tabla equipamiento
    if let Err(e) = conn
        .execute(
            "UPDATE equipamiento SET stock_disponible = stock_disponible + ?1 WHERE id = ?2",
            (cantidad, equipamiento_id),
        )
        .await
    {
        let _ = conn.execute("ROLLBACK", ()).await;
        let error_html = crate::components::alerta::alerta(
            "error",
            "Error de Reabastecimiento de Stock",
            &e.to_string(),
        );
        let fragmento =
            render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
    }

    // 8. Completar transacción
    if let Err(e) = conn.execute("COMMIT", ()).await {
        let _ = conn.execute("ROLLBACK", ()).await;
        let error_html =
            crate::components::alerta::alerta("error", "Error de Commit", &e.to_string());
        let fragmento =
            render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
    }

    tracing::info!(
        operador_id = %operador.id,
        equipo = %equipo_nombre,
        cantidad = %cantidad,
        "Devolución de equipamiento procesada con éxito"
    );

    let exito_html = crate::components::alerta::alerta(
        "success",
        "Devolución Completada",
        "El equipamiento ha sido devuelto a la armería y el stock disponible se ha reintegrado.",
    );
    let fragmento =
        render_fragmento_asignaciones(&state, &jar, &csrf_token.0, Some(exito_html)).await;
    fragmento.into_response()
}

/// Handler GET `/asignaciones/auditoria`: Verifica la integridad de la cadena de custodia (Hash-Chain).
/// Retorna un fragmento de UI diseñado para reemplazar el div #resultado-auditoria vía HTMX.
pub async fn auditar_bitacora(
    State(state): State<AppState>,
    jar: axum_extra::extract::PrivateCookieJar,
) -> Response {
    // 1. Verificamos que haya una sesión activa antes de auditar
    if resolver_operador_activo(&jar, &state)
        .await
        .unwrap_or(None)
        .is_none()
    {
        return (
            StatusCode::FORBIDDEN,
            crate::components::alerta::alerta("error", "Acceso Denegado", "Sesión inválida."),
        )
            .into_response();
    }

    // 2. Conectamos a la BD
    let conn = match state.db.connect() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                crate::components::alerta::alerta("error", "Error de Conexión", &e.to_string()),
            )
                .into_response();
        }
    };

    // 3. Ejecutar la auditoría criptográfica
    match crate::security::verificar_integridad_bitacora(&conn).await {
        Ok(resultado) => {
            if resultado.cadena_integra {
                let msg = format!(
                    "Se validaron {} registros. Ningún evento ha sido alterado.",
                    resultado.total_registros
                );
                crate::components::alerta::alerta("success", "CADENA DE CUSTODIA ÍNTEGRA", &msg)
                    .into_response()
            } else {
                let msg = format!(
                    "¡ATENCIÓN! La integridad de la bitácora ha sido comprometida a partir del Registro ID: {}. La cadena de custodia carece de validez legal a partir de ese punto.",
                    resultado.primer_registro_corrupto.unwrap_or(0)
                );
                crate::components::alerta::alerta(
                    "error",
                    "VIOLACIÓN DE INTEGRIDAD DETECTADA",
                    &msg,
                )
                .into_response()
            }
        }
        Err(e) => crate::components::alerta::alerta("error", "Error durante Auditoría", &e)
            .into_response(),
    }
}
