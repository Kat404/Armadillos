#![allow(dead_code)]
use crate::AppState;
use crate::layouts::main_layout::{PageContext, layout};
use crate::pages::soldados::resolver_operador_activo;
use crate::security::CsrfToken;
use axum::{
    Extension, Form,
    extract::Query,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use maud::{Markup, html};
use serde::Deserialize;

/// Representación mínima de un soldado para el selector de operador simulado.
pub struct SoldadoOpcion {
    pub id: i64,
    pub nombre: String,
    pub apellido: String,
    pub rango_nombre: String,
}

/// Obtiene todos los soldados para poblar el selector de simulación.
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

/// Representación interna de un ítem de equipamiento para listar en la UI.
pub struct ItemEquipamiento {
    pub id: i64,
    pub codigo_inventario: String,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub categoria_id: i64,
    pub categoria_nombre: String,
    pub es_material_de_guerra: bool,
    pub estado_conservacion: String,
    pub stock_total: i64,
    pub stock_disponible: i64,
}

/// Recupera los ítems de equipamiento, con opción de filtrado por categoría.
async fn obtener_equipamiento(
    state: &AppState,
    categoria_id_opt: Option<i64>,
) -> Result<Vec<ItemEquipamiento>, String> {
    let conn = state.db.connect().map_err(|e| e.to_string())?;
    let mut rows = if let Some(cat_id) = categoria_id_opt {
        conn.query(
            "SELECT e.id, e.codigo_inventario, e.nombre, e.descripcion, e.categoria_id, c.nombre, c.es_material_de_guerra, e.estado_conservacion, e.stock_total, e.stock_disponible
             FROM equipamiento e
             JOIN categorias_equipamiento c ON e.categoria_id = c.id
             WHERE e.categoria_id = ?1
             ORDER BY e.nombre ASC",
            (cat_id,),
        )
        .await
        .map_err(|e| e.to_string())?
    } else {
        conn.query(
            "SELECT e.id, e.codigo_inventario, e.nombre, e.descripcion, e.categoria_id, c.nombre, c.es_material_de_guerra, e.estado_conservacion, e.stock_total, e.stock_disponible
             FROM equipamiento e
             JOIN categorias_equipamiento c ON e.categoria_id = c.id
             ORDER BY c.nombre ASC, e.nombre ASC",
            (),
        )
        .await
        .map_err(|e| e.to_string())?
    };

    let mut lista = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let es_material_de_guerra_num: i64 = row.get(6).map_err(|e| e.to_string())?;
        lista.push(ItemEquipamiento {
            id: row.get(0).map_err(|e| e.to_string())?,
            codigo_inventario: row.get(1).map_err(|e| e.to_string())?,
            nombre: row.get(2).map_err(|e| e.to_string())?,
            descripcion: row.get(3).ok(),
            categoria_id: row.get(4).map_err(|e| e.to_string())?,
            categoria_nombre: row.get(5).map_err(|e| e.to_string())?,
            es_material_de_guerra: es_material_de_guerra_num != 0,
            estado_conservacion: row.get(7).map_err(|e| e.to_string())?,
            stock_total: row.get(8).map_err(|e| e.to_string())?,
            stock_disponible: row.get(9).map_err(|e| e.to_string())?,
        });
    }
    Ok(lista)
}

/// Representación mínima de una categoría para poblar filtros y selects.
pub struct CategoriaOpcion {
    pub id: i64,
    pub nombre: String,
    pub es_material_de_guerra: bool,
}

async fn obtener_categorias(state: &AppState) -> Result<Vec<CategoriaOpcion>, String> {
    let conn = state.db.connect().map_err(|e| e.to_string())?;
    let mut rows = conn
        .query("SELECT id, nombre, es_material_de_guerra FROM categorias_equipamiento ORDER BY nombre ASC", ())
        .await
        .map_err(|e| e.to_string())?;
    let mut lista = Vec::new();
    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        let es_material_de_guerra_num: i64 = row.get(2).map_err(|e| e.to_string())?;
        lista.push(CategoriaOpcion {
            id: row.get(0).map_err(|e| e.to_string())?,
            nombre: row.get(1).map_err(|e| e.to_string())?,
            es_material_de_guerra: es_material_de_guerra_num != 0,
        });
    }
    Ok(lista)
}

#[derive(Deserialize)]
pub struct FiltroInventario {
    pub categoria_id: Option<i64>,
}

/// Renders the inner content of the inventory page, which can be swapped asynchronously by HTMX.
async fn render_fragmento_inventario(
    state: &AppState,
    headers: &HeaderMap,
    csrf_token: &str,
    categoria_id_opt: Option<i64>,
    alerta_markup: Option<Markup>,
) -> Markup {
    let operador = resolver_operador_activo(headers, state)
        .await
        .unwrap_or(None);
    let todos_soldados = obtener_soldados_opciones(state).await.unwrap_or_default();
    let categorias = obtener_categorias(state).await.unwrap_or_default();
    let inventario = obtener_equipamiento(state, categoria_id_opt)
        .await
        .unwrap_or_default();

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
        div id="seccion-inventario-completo" {
            // 1. Selector de Operador (Simulador ABAC componentizado)
            (crate::components::operador::selector_operador(
                op_id,
                op_nombre,
                op_rango,
                op_seccion,
                puede_belico,
                &soldados_tuple,
                "/inventario",
            ))

            // 2. Banner de alertas inyectado dinámicamente si existe
            @if let Some(alerta) = alerta_markup {
                (alerta)
            }

            div class="grid" {
                // Columna Izquierda: Formulario de Registro (CRUD Create)
                div class="s12 m4" {
                    article class="border round medium-padding" {
                        h5 class="medium-margin" { "Registrar Equipamiento" }

                        form hx-post="/inventario" hx-target="#seccion-inventario-completo" hx-swap="outerHTML" {
                            input type="hidden" name="csrf_token" value=(csrf_token);

                            div class="field label border" {
                                input type="text" id="codigo_inventario" name="codigo_inventario" placeholder=" " required;
                                label for="codigo_inventario" { "Código de Inventario (ej. ARM-100)" }
                            }

                            div class="field label border" {
                                input type="text" id="nombre" name="nombre" placeholder=" " required;
                                label for="nombre" { "Nombre del Equipo" }
                            }

                            div class="field label border" {
                                textarea id="descripcion" name="descripcion" placeholder=" " {};
                                label for="descripcion" { "Descripción Técnica" }
                            }

                            div class="field label border" {
                                select id="categoria_id" name="categoria_id" required {
                                    @for c in &categorias {
                                        @let no_permitido = c.es_material_de_guerra && !puede_belico;
                                        option value=(c.id) disabled?[no_permitido] {
                                            (c.nombre)
                                            @if c.es_material_de_guerra {
                                                @if no_permitido { " (⚠ Bélico — Bloqueado por ABAC)" } @else { " (Material de Guerra ⚠)" }
                                            }
                                        }
                                    }
                                }
                                label for="categoria_id" { "Categoría" }
                            }

                            div class="field label border" {
                                select id="estado_conservacion" name="estado_conservacion" required {
                                    option value="Operativo" { "Operativo" }
                                    option value="En Mantenimiento" { "En Mantenimiento" }
                                    option value="De Baja" { "De Baja" }
                                }
                                label for="estado_conservacion" { "Estado de Conservación" }
                            }

                            div class="field label border" {
                                input type="number" id="stock_total" name="stock_total" min="1" value="1" placeholder=" " required;
                                label for="stock_total" { "Cantidad Inicial (Stock)" }
                            }

                            button type="submit" class="responsive primary" {
                                i { "save" }
                                span { "Guardar Registro" }
                            }
                        }
                    }
                }

                // Columna Derecha: Tabla de Equipamiento con filtros
                div class="s12 m8" {
                    article class="border round medium-padding" {
                        div class="row align-center margin-bottom" {
                            div class="max" {
                                h5 class="no-margin" { "Inventario Disponible" }
                            }
                            div class="col" {
                                // Filtros por Categoría
                                form method="GET" action="/inventario" class="row gap align-center no-margin" {
                                    div class="field label border small no-margin" {
                                        select name="categoria_id" onchange="this.form.submit()" {
                                            option value="" selected?[categoria_id_opt.is_none()] { "Todas las Categorías" }
                                            @for c in &categorias {
                                                option value=(c.id) selected?[categoria_id_opt == Some(c.id)] { (c.nombre) }
                                            }
                                        }
                                        label { "Filtrar por" }
                                    }
                                }
                            }
                        }

                        div class="table-container" {
                            table class="striped hover" {
                                thead {
                                    tr {
                                        th { "Código" }
                                        th { "Nombre / Descripción" }
                                        th { "Categoría" }
                                        th { "Estado" }
                                        th { "Stock Total" }
                                        th { "Disponible" }
                                    }
                                }
                                tbody {
                                    @if inventario.is_empty() {
                                        tr {
                                            td colspan="6" class="center-align text-secondary" { "No hay equipamiento registrado en esta categoría." }
                                        }
                                    }
                                    @for item in &inventario {
                                        tr {
                                            td class="bold font-mono" { (item.codigo_inventario) }
                                            td {
                                                div class="bold" {
                                                    (item.nombre)
                                                    @if item.es_material_de_guerra {
                                                        span class="badge none red circle" style="margin-left: 8px" title="Material de Guerra Sensible" { "⚠ Bélico" }
                                                    }
                                                }
                                                @if let Some(ref desc) = item.descripcion {
                                                    div class="caption text-secondary" { (desc) }
                                                }
                                            }
                                            td { (item.categoria_nombre) }
                                            td {
                                                @if item.estado_conservacion == "Operativo" {
                                                    span class="text-success bold" { "Operativo" }
                                                } @else if item.estado_conservacion == "En Mantenimiento" {
                                                    span class="text-warning bold" { "En Mantenimiento" }
                                                } @else {
                                                    span class="text-error bold" { "De Baja" }
                                                }
                                            }
                                            td class="center-align" { (item.stock_total) }
                                            td class="center-align bold" {
                                                @if item.stock_disponible == 0 {
                                                    span class="text-error" { "Agotado" }
                                                } @else if item.stock_disponible <= 5 {
                                                    span class="text-warning" { (item.stock_disponible) }
                                                } @else {
                                                    span class="text-success" { (item.stock_disponible) }
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

/// Handler GET `/inventario`: Muestra la página de administración de equipamiento.
pub async fn pagina_inventario(
    State(state): State<AppState>,
    Extension(csrf_token): Extension<CsrfToken>,
    headers: HeaderMap,
    Query(filtro): Query<FiltroInventario>,
) -> Markup {
    let ctx = PageContext::new(
        "Armadillos - Inventario Militar",
        "Inventario y Equipamiento",
        "Panel central para el registro, consulta y control del equipamiento militar con autorización ABAC.",
        &csrf_token.0,
    );

    let fragmento =
        render_fragmento_inventario(&state, &headers, &csrf_token.0, filtro.categoria_id, None)
            .await;

    layout(&ctx, fragmento)
}

/// Estructura que captura los datos enviados para registrar un equipamiento.
#[derive(Deserialize)]
pub struct NuevoEquipamiento {
    pub codigo_inventario: String,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub categoria_id: i64,
    pub estado_conservacion: String,
    pub stock_total: i64,
}

/// Handler POST `/inventario`: Agrega equipamiento a la BD previa validación ABAC, retornando la vista parcial actualizada.
pub async fn agregar_equipamiento(
    State(state): State<AppState>,
    Extension(csrf_token): Extension<CsrfToken>,
    headers: HeaderMap,
    Form(payload): Form<NuevoEquipamiento>,
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
            let fragmento = render_fragmento_inventario(
                &state,
                &headers,
                &csrf_token.0,
                None,
                Some(error_html),
            )
            .await;
            return (StatusCode::FORBIDDEN, fragmento).into_response();
        }
    };

    // 2. Determinar si la categoría elegida es Material de Guerra
    let conn = match state.db.connect() {
        Ok(c) => c,
        Err(e) => {
            let error_html =
                crate::components::alerta::alerta("error", "Fallo de Conexión", &e.to_string());
            let fragmento = render_fragmento_inventario(
                &state,
                &headers,
                &csrf_token.0,
                None,
                Some(error_html),
            )
            .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
        }
    };

    let mut rows = match conn
        .query(
            "SELECT es_material_de_guerra FROM categorias_equipamiento WHERE id = ?1",
            (payload.categoria_id,),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            let error_html =
                crate::components::alerta::alerta("error", "Error de Consulta", &e.to_string());
            let fragmento = render_fragmento_inventario(
                &state,
                &headers,
                &csrf_token.0,
                None,
                Some(error_html),
            )
            .await;
            return (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response();
        }
    };

    let es_material_de_guerra = match rows.next().await {
        Ok(Some(row)) => {
            let val: i64 = row.get(0).unwrap_or(0);
            val != 0
        }
        _ => false,
    };

    // 3. Evaluar la política ABAC si es Material de Guerra
    if es_material_de_guerra && !operador.contexto.puede_administrar_inventario_belico() {
        tracing::warn!(
            operador = %operador.nombre_completo,
            rango = %operador.rango_nombre,
            "Intento bloqueado por ABAC: usuario no autorizado intentó registrar material de guerra"
        );
        let error_msg = format!(
            "El operador simulado ({} — {}) carece de rango de Oficial o de adscripción al Servicio de Materiales de Guerra para dar de alta armas o munición.",
            operador.rango_nombre, operador.nombre_completo
        );
        let error_html =
            crate::components::alerta::alerta("error", "PERMISO DENEGADO (ABAC)", &error_msg);
        let fragmento =
            render_fragmento_inventario(&state, &headers, &csrf_token.0, None, Some(error_html))
                .await;
        return (StatusCode::FORBIDDEN, fragmento).into_response();
    }

    // 4. Ejecutar la inserción en la base de datos
    let desc = payload
        .descripcion
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string());

    match conn
        .execute(
            "INSERT INTO equipamiento (codigo_inventario, nombre, descripcion, categoria_id, estado_conservacion, stock_total, stock_disponible)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                payload.codigo_inventario,
                payload.nombre,
                desc,
                payload.categoria_id,
                payload.estado_conservacion,
                payload.stock_total,
                payload.stock_total,
            ),
        )
        .await
    {
        Ok(_) => {
            tracing::info!(
                operador = %operador.nombre_completo,
                "Equipamiento registrado con éxito en el inventario"
            );
            // Éxito: retornamos la vista parcial limpia
            let exito_html = crate::components::alerta::alerta("success", "Registro Exitoso", "El nuevo equipamiento ha sido incorporado al inventario con éxito.");
            let fragmento = render_fragmento_inventario(&state, &headers, &csrf_token.0, None, Some(exito_html)).await;
            fragmento.into_response()
        }
        Err(e) => {
            let error_html = crate::components::alerta::alerta("error", "Error de Inserción SQL", &e.to_string());
            let fragmento = render_fragmento_inventario(&state, &headers, &csrf_token.0, None, Some(error_html)).await;
            (StatusCode::INTERNAL_SERVER_ERROR, fragmento).into_response()
        }
    }
}
