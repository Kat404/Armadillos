#![allow(dead_code)]
use crate::AppState;
use crate::layouts::main_layout::{PageContext, layout};
use crate::pages::inventario::SoldadoOpcion;
use crate::pages::soldados::resolver_operador_activo;
use crate::security::CsrfToken;
use axum::{
    Extension, Form,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use maud::{Markup, html};
use serde::Deserialize;

/// Representación para el historial de cadena de custodia.
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
            "SELECT s.id, s.nombre, s.apellido_paterno, r.nombre
             FROM soldados s
             JOIN rangos r ON s.rango_id = r.id
             ORDER BY r.orden_jerarquico DESC, s.nombre ASC",
            (),
        )
        .await
        .map_err(|e| e.to_string())?;
    let mut lista = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        lista.push(SoldadoOpcion {
            id: row.get(0).map_err(|e| e.to_string())?,
            nombre: row.get(1).map_err(|e| e.to_string())?,
            apellido: row.get(2).map_err(|e| e.to_string())?,
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

/// Obtiene el listado completo del historial de asignaciones.
async fn obtener_asignaciones(state: &AppState) -> Result<Vec<DetalleAsignacion>, String> {
    let conn = state.db.connect().map_err(|e| e.to_string())?;
    let mut rows = conn
        .query(
            "SELECT a.id, a.fecha_asignacion, a.cantidad,
                    s_rec.nombre || ' ' || s_rec.apellido_paterno AS receptor_nombre,
                    r_rec.nombre AS receptor_rango,
                    e.nombre AS equipo_nombre,
                    e.codigo_inventario AS equipo_codigo,
                    c.nombre AS categoria_nombre,
                    c.es_material_de_guerra AS es_belico,
                    s_aut.nombre || ' ' || s_aut.apellido_paterno AS autorizante_nombre,
                    r_aut.nombre AS autorizante_rango
             FROM asignaciones_equipamiento a
             JOIN soldados s_rec ON a.soldado_id = s_rec.id
             JOIN rangos r_rec ON s_rec.rango_id = r_rec.id
             JOIN equipamiento e ON a.equipamiento_id = e.id
             JOIN categorias_equipamiento c ON e.categoria_id = c.id
             JOIN soldados s_aut ON a.autorizado_por_soldado_id = s_aut.id
             JOIN rangos r_aut ON s_aut.rango_id = r_aut.id
             ORDER BY a.fecha_asignacion DESC",
            (),
        )
        .await
        .map_err(|e| e.to_string())?;

    let mut lista = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let es_belico_num: i64 = row.get(8).map_err(|e| e.to_string())?;
        lista.push(DetalleAsignacion {
            id: row.get(0).map_err(|e| e.to_string())?,
            fecha_asignacion: row.get(1).map_err(|e| e.to_string())?,
            cantidad: row.get(2).map_err(|e| e.to_string())?,
            receptor_nombre: row.get(3).map_err(|e| e.to_string())?,
            receptor_rango: row.get(4).map_err(|e| e.to_string())?,
            equipo_nombre: row.get(5).map_err(|e| e.to_string())?,
            equipo_codigo: row.get(6).map_err(|e| e.to_string())?,
            categoria_nombre: row.get(7).map_err(|e| e.to_string())?,
            es_belico: es_belico_num != 0,
            autorizante_nombre: row.get(9).map_err(|e| e.to_string())?,
            autorizante_rango: row.get(10).map_err(|e| e.to_string())?,
        });
    }
    Ok(lista)
}

/// Renders the inner content of the assignments page, which can be swapped asynchronously by HTMX.
async fn render_fragmento_asignaciones(
    state: &AppState,
    headers: &HeaderMap,
    csrf_token: &str,
    alerta_markup: Option<Markup>,
) -> Markup {
    let operador = resolver_operador_activo(headers, state)
        .await
        .unwrap_or(None);
    let todos_soldados = obtener_soldados_opciones(state).await.unwrap_or_default();
    let equipamiento = obtener_equipamiento_opciones(state)
        .await
        .unwrap_or_default();
    let asignaciones = obtener_asignaciones(state).await.unwrap_or_default();

    let soldados_tuple: Vec<(i64, String)> = todos_soldados
        .iter()
        .map(|s| {
            (
                s.id,
                format!("{} — {} {}", s.rango_nombre, s.nombre, s.apellido),
            )
        })
        .collect();

    let op_id = operador.as_ref().map(|o| o.id);
    let op_nombre = operador.as_ref().map(|o| o.nombre_completo.clone());
    let op_rango = operador.as_ref().map(|o| o.rango_nombre.clone());
    let op_seccion = operador.as_ref().map(|o| o.seccion_nombre.clone());
    let puede_belico = operador
        .as_ref()
        .map(|o| o.contexto.puede_administrar_inventario_belico())
        .unwrap_or(false);

    html! {
        div id="seccion-asignaciones-completo" {
            // 1. Selector de Operador (Simulador ABAC componentizado)
            (crate::components::operador::selector_operador(
                op_id,
                op_nombre,
                op_rango,
                op_seccion,
                puede_belico,
                &soldados_tuple,
                "/asignaciones",
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
                        h5 class="medium-margin" { "Registro Histórico de Entregas" }

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
                                                @if asig.cantidad > 0 {
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
    headers: HeaderMap,
) -> Markup {
    let ctx = PageContext::new(
        "Armadillos - Asignaciones",
        "Cadena de Custodia de Equipamiento",
        "Registro y control del armamento y equipo asignado individualmente al personal militar.",
        &csrf_token.0,
    );

    let fragmento = render_fragmento_asignaciones(&state, &headers, &csrf_token.0, None).await;

    layout(&ctx, fragmento)
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
    headers: HeaderMap,
    Form(payload): Form<NuevaAsignacion>,
) -> Response {
    // 1. Resolver el Operador Activo
    let operador = match resolver_operador_activo(&headers, &state).await {
        Ok(Some(op)) => op,
        _ => {
            let error_html = crate::components::alerta::alerta(
                "error",
                "Simulación Inválida",
                "No hay un operador simulado activo o válido.",
            );
            let fragmento =
                render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html))
                    .await;
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
                render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html))
                    .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
        }
    };

    // 3. Iniciar Transacción Manual para asegurar consistencia ACID
    if let Err(e) = conn.execute("BEGIN TRANSACTION", ()).await {
        let error_html =
            crate::components::alerta::alerta("error", "Error de Transacción", &e.to_string());
        let fragmento =
            render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html)).await;
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
                let fragmento = render_fragmento_asignaciones(
                    &state,
                    &headers,
                    &csrf_token.0,
                    Some(error_html),
                )
                .await;
                return (StatusCode::BAD_REQUEST, fragmento).into_response();
            }
        },
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ()).await;
            let error_html =
                crate::components::alerta::alerta("error", "Error de Consulta SQL", &e.to_string());
            let fragmento =
                render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html))
                    .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
        }
    };

    // 5. Validar ABAC para material de guerra
    if es_material_de_guerra && !operador.contexto.puede_administrar_inventario_belico() {
        let _ = conn.execute("ROLLBACK", ()).await;
        tracing::warn!(
            operador = %operador.nombre_completo,
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
            render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html)).await;
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
            render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::BAD_REQUEST, fragmento).into_response();
    }

    // 7. Insertar el registro de asignación
    if let Err(e) = conn
        .execute(
            "INSERT INTO asignaciones_equipamiento (soldado_id, equipamiento_id, cantidad, autorizado_por_soldado_id)
             VALUES (?1, ?2, ?3, ?4)",
            (
                payload.soldado_id,
                payload.equipamiento_id,
                payload.cantidad,
                operador.id,
            ),
        )
        .await
    {
        let _ = conn.execute("ROLLBACK", ()).await;
        let error_html = crate::components::alerta::alerta("error", "Error de Inserción SQL", &e.to_string());
        let fragmento = render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html)).await;
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
            render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
    }

    // 9. Completar la transacción
    if let Err(e) = conn.execute("COMMIT", ()).await {
        let _ = conn.execute("ROLLBACK", ()).await;
        let error_html =
            crate::components::alerta::alerta("error", "Error de Commit", &e.to_string());
        let fragmento =
            render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
    }

    tracing::info!(
        operador = %operador.nombre_completo,
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
        render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(exito_html)).await;
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
    headers: HeaderMap,
    Form(payload): Form<DevolucionAsignacion>,
) -> Response {
    // 1. Resolver el Operador Activo (quien procesa el retorno de armamento)
    let operador = match resolver_operador_activo(&headers, &state).await {
        Ok(Some(op)) => op,
        _ => {
            let error_html = crate::components::alerta::alerta(
                "error",
                "Simulación Inválida",
                "No hay un operador simulado activo o válido.",
            );
            let fragmento =
                render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html))
                    .await;
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
                render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html))
                    .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
        }
    };

    // 3. Consultar datos de la asignación existente para conocer cantidad y equipo
    let mut rows = match conn
        .query(
            "SELECT a.equipamiento_id, a.cantidad, c.es_material_de_guerra, e.nombre
             FROM asignaciones_equipamiento a
             JOIN equipamiento e ON a.equipamiento_id = e.id
             JOIN categorias_equipamiento c ON e.categoria_id = c.id
             WHERE a.id = ?1",
            (payload.asignacion_id,),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            let error_html =
                crate::components::alerta::alerta("error", "Error de Consulta SQL", &e.to_string());
            let fragmento =
                render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html))
                    .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
        }
    };

    let (equipamiento_id, cantidad, es_belico, equipo_nombre) = match rows.next().await {
        Ok(Some(row)) => {
            let eq_id: i64 = row.get(0).unwrap();
            let cant: i64 = row.get(1).unwrap();
            let belico_num: i64 = row.get(2).unwrap();
            let nombre: String = row.get(3).unwrap();
            (eq_id, cant, belico_num != 0, nombre)
        }
        _ => {
            let error_html = crate::components::alerta::alerta(
                "error",
                "Asignación Inexistente",
                "No se encontró el registro de asignación especificado.",
            );
            let fragmento =
                render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html))
                    .await;
            return (StatusCode::BAD_REQUEST, fragmento).into_response();
        }
    };

    // 4. Evaluar la política ABAC si el equipo a devolver es Material de Guerra
    if es_belico && !operador.contexto.puede_administrar_inventario_belico() {
        tracing::warn!(
            operador = %operador.nombre_completo,
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
            render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::FORBIDDEN, fragmento).into_response();
    }

    // 5. Iniciar la transacción SQLite
    if let Err(e) = conn.execute("BEGIN TRANSACTION", ()).await {
        let error_html =
            crate::components::alerta::alerta("error", "Error de Transacción", &e.to_string());
        let fragmento =
            render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
    }

    // 6. Eliminar el registro de la asignación (o poner cantidad = 0)
    // Para simplificar y mantener el registro en el historial visual (marcado como devuelto),
    // actualizaremos la cantidad asignada a 0 en la bitácora.
    if let Err(e) = conn
        .execute(
            "UPDATE asignaciones_equipamiento SET cantidad = 0 WHERE id = ?1",
            (payload.asignacion_id,),
        )
        .await
    {
        let _ = conn.execute("ROLLBACK", ()).await;
        let error_html = crate::components::alerta::alerta(
            "error",
            "Error de Actualización de Asignación",
            &e.to_string(),
        );
        let fragmento =
            render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html)).await;
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
            render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
    }

    // 8. Completar transacción
    if let Err(e) = conn.execute("COMMIT", ()).await {
        let _ = conn.execute("ROLLBACK", ()).await;
        let error_html =
            crate::components::alerta::alerta("error", "Error de Commit", &e.to_string());
        let fragmento =
            render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(error_html)).await;
        return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
    }

    tracing::info!(
        operador = %operador.nombre_completo,
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
        render_fragmento_asignaciones(&state, &headers, &csrf_token.0, Some(exito_html)).await;
    fragmento.into_response()
}
