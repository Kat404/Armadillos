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

fn extract_cookie(req: &Request<Body>, name: &str) -> Option<String> {
    req.headers()
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .map(|s| s.trim())
        .find(|s| s.starts_with(name))
        .and_then(|s| s.split('=').nth(1))
        .map(|s| s.to_string())
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
}
