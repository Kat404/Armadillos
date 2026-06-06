use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    middleware::Next,
    response::Response,
};
use rand::{RngCore, rngs::OsRng};

#[derive(Clone, Debug)]
pub struct CsrfToken(pub String);

pub fn generate_csrf_token() -> String {
    let mut token_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut token_bytes);
    token_bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn extraer_cookie(headers: &header::HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .map(|s| s.trim())
        .find(|s| s.starts_with(name))
        .and_then(|s| s.split('=').nth(1))
        .map(|s| s.to_string())
}

fn extract_cookie(req: &Request<Body>, name: &str) -> Option<String> {
    extraer_cookie(req.headers(), name)
}

pub async fn csrf_middleware(mut req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    // 1. Obtener el token de la cookie
    let cookie_token = extract_cookie(&req, "__Host-csrf");

    // 2. Si es una petición insegura (POST/PUT/DELETE), validar el token
    let method = req.method();
    if method == axum::http::Method::POST
        || method == axum::http::Method::PUT
        || method == axum::http::Method::DELETE
        || method == axum::http::Method::PATCH
    {
        let header_token = req
            .headers()
            .get("X-CSRF-Token")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        match (cookie_token.as_ref(), header_token.as_ref()) {
            (Some(c), Some(h)) if c == h && !c.is_empty() => {
                // Token válido, procedemos
            }
            _ => {
                // Validación fallida, bloqueamos con 403 Forbidden
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }

    // 3. Obtener o generar token de sesión
    let token = cookie_token.unwrap_or_else(generate_csrf_token);

    // 4. Guardar token en las extensiones del request para los handlers
    req.extensions_mut().insert(CsrfToken(token.clone()));

    // 5. Proceder con el request
    let mut response = next.run(req).await;

    // 6. Establecer la cookie __Host-csrf
    // Directivas: Secure, HttpOnly, SameSite=Strict, Path=/
    let cookie_header = format!(
        "__Host-csrf={}; Path=/; Secure; HttpOnly; SameSite=Strict",
        token
    );

    response.headers_mut().append(
        header::SET_COOKIE,
        header::HeaderValue::from_str(&cookie_header).unwrap(),
    );

    Ok(response)
}

// ============================================================
// Hash-Chain BLAKE3 para no-repudio en la cadena de custodia
// ============================================================

/// Hash génesis: el "bloque cero" de la cadena de custodia.
/// Representa el hash previo cuando la bitácora está vacía.
pub const HASH_GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Calcula el hash BLAKE3 de un evento de la cadena de custodia.
///
/// La cadena a hashear se construye concatenando los campos del registro
/// con el hash del registro inmediatamente anterior, siguiendo el patrón:
/// `hash_actual = BLAKE3(tipo_evento|soldado_id|equipamiento_id|cantidad|fecha_evento|autorizado_por_soldado_id|asignacion_origen_id|hash_anterior)`
///
/// **¿Por qué BLAKE3?** Es ~2.5-9x más rápido que SHA-256 en software puro,
/// inmune a ataques de extensión de longitud y libre de patentes (CC0/Apache 2.0).
#[allow(clippy::too_many_arguments)]
pub fn calcular_hash_evento(
    tipo_evento: &str,
    soldado_id: i64,
    equipamiento_id: i64,
    cantidad: i64,
    fecha_evento: &str,
    autorizado_por_soldado_id: i64,
    asignacion_origen_id: Option<i64>,
    hash_anterior: &str,
) -> String {
    let origen_str = asignacion_origen_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "NULL".to_string());

    let datos = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        tipo_evento,
        soldado_id,
        equipamiento_id,
        cantidad,
        fecha_evento,
        autorizado_por_soldado_id,
        origen_str,
        hash_anterior,
    );

    blake3::hash(datos.as_bytes()).to_hex().to_string()
}

/// Resultado de la auditoría de integridad de la cadena de custodia.
#[allow(dead_code)]
#[derive(Debug)]
pub struct ResultadoAuditoria {
    /// Total de registros verificados.
    pub total_registros: usize,
    /// Verdadero si toda la cadena es íntegra.
    pub cadena_integra: bool,
    /// Si la cadena fue comprometida, el ID del primer registro corrupto.
    pub primer_registro_corrupto: Option<i64>,
}

/// Verifica la integridad de toda la cadena de custodia recorriendo
/// secuencialmente la tabla `asignaciones_equipamiento`, recalculando
/// cada hash y comparándolo con el almacenado en la BD.
#[allow(dead_code)]
pub async fn verificar_integridad_bitacora(
    conn: &turso::Connection,
) -> Result<ResultadoAuditoria, String> {
    let mut rows = conn
        .query(
            "SELECT id, tipo_evento, soldado_id, equipamiento_id, cantidad,
                    fecha_evento, autorizado_por_soldado_id, asignacion_origen_id,
                    hash_verificacion
             FROM asignaciones_equipamiento
             ORDER BY id ASC",
            (),
        )
        .await
        .map_err(|e| e.to_string())?;

    let mut hash_anterior = HASH_GENESIS.to_string();
    let mut total_registros: usize = 0;

    while let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        total_registros += 1;
        let id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let tipo_evento: String = row.get(1).map_err(|e| e.to_string())?;
        let soldado_id: i64 = row.get(2).map_err(|e| e.to_string())?;
        let equipamiento_id: i64 = row.get(3).map_err(|e| e.to_string())?;
        let cantidad: i64 = row.get(4).map_err(|e| e.to_string())?;
        let fecha_evento: String = row.get(5).map_err(|e| e.to_string())?;
        let autorizado_por: i64 = row.get(6).map_err(|e| e.to_string())?;
        let origen_id: Option<i64> = row.get::<i64>(7).ok();
        let hash_almacenado: String = row.get(8).map_err(|e| e.to_string())?;

        let hash_calculado = calcular_hash_evento(
            &tipo_evento,
            soldado_id,
            equipamiento_id,
            cantidad,
            &fecha_evento,
            autorizado_por,
            origen_id,
            &hash_anterior,
        );

        if hash_calculado != hash_almacenado {
            tracing::error!(
                registro_id = id,
                hash_esperado = %hash_calculado,
                hash_almacenado = %hash_almacenado,
                "⚠️ ALERTA DE INTEGRIDAD: Hash-Chain roto en la cadena de custodia"
            );
            return Ok(ResultadoAuditoria {
                total_registros,
                cadena_integra: false,
                primer_registro_corrupto: Some(id),
            });
        }

        hash_anterior = hash_almacenado;
    }

    Ok(ResultadoAuditoria {
        total_registros,
        cadena_integra: true,
        primer_registro_corrupto: None,
    })
}

/// Obtiene el hash del último registro de la cadena de custodia.
/// Si la tabla está vacía, retorna el hash génesis.
pub async fn obtener_ultimo_hash(conn: &turso::Connection) -> Result<String, String> {
    let mut rows = conn
        .query(
            "SELECT hash_verificacion FROM asignaciones_equipamiento ORDER BY id DESC LIMIT 1",
            (),
        )
        .await
        .map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        row.get::<String>(0).map_err(|e| e.to_string())
    } else {
        Ok(HASH_GENESIS.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csrf_token_generation() {
        let t1 = generate_csrf_token();
        let t2 = generate_csrf_token();

        // Verificar que los tokens tengan 64 caracteres hex (32 bytes)
        assert_eq!(t1.len(), 64);
        assert_eq!(t2.len(), 64);

        // Verificar que sean aleatorios y únicos
        assert_ne!(t1, t2);

        // Verificar que sean hex
        assert!(t1.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(t2.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_evento_determinista() {
        let h1 = calcular_hash_evento(
            "ASIGNACION",
            3,
            1,
            2,
            "2026-06-06 12:00:00",
            1,
            None,
            HASH_GENESIS,
        );
        let h2 = calcular_hash_evento(
            "ASIGNACION",
            3,
            1,
            2,
            "2026-06-06 12:00:00",
            1,
            None,
            HASH_GENESIS,
        );
        // Mismos datos → mismo hash
        assert_eq!(h1, h2);
        // Hash BLAKE3 de 64 caracteres hex
        assert_eq!(h1.len(), 64);
        assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_evento_sensible_a_cambios() {
        let h_original = calcular_hash_evento(
            "ASIGNACION",
            3,
            1,
            2,
            "2026-06-06 12:00:00",
            1,
            None,
            HASH_GENESIS,
        );
        // Cambiar cantidad de 2 a 3 → hash diferente
        let h_alterado = calcular_hash_evento(
            "ASIGNACION",
            3,
            1,
            3,
            "2026-06-06 12:00:00",
            1,
            None,
            HASH_GENESIS,
        );
        assert_ne!(h_original, h_alterado);
    }

    #[test]
    fn test_hash_chain_encadenamiento() {
        let h1 = calcular_hash_evento(
            "ASIGNACION",
            3,
            1,
            2,
            "2026-06-06 12:00:00",
            1,
            None,
            HASH_GENESIS,
        );
        let h2 = calcular_hash_evento(
            "DEVOLUCION",
            3,
            1,
            2,
            "2026-06-06 13:00:00",
            1,
            Some(1),
            &h1,
        );
        // Alterar h1 rompe la cadena: h2 calculado con h1 alterado sería diferente
        let h1_falso = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let h2_falso = calcular_hash_evento(
            "DEVOLUCION",
            3,
            1,
            2,
            "2026-06-06 13:00:00",
            1,
            Some(1),
            h1_falso,
        );
        assert_ne!(h2, h2_falso);
    }
}
