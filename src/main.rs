mod components; // Declaramos módulo de componentes/
mod crypto;
mod db; // Módulo de base de datos (migraciones y esquema)
mod domain; // Módulo para tipos seguros de dominio militar
mod layouts; // Declaramos módulo raíz de los layouts/
mod pages; // Declaramos módulo raíz de las pages/
mod security; // Módulo de seguridad (CSRF) // Módulo de criptografía (ALE)

use axum::{Router, routing::get};
use pages::about::pagina_about; // Importamos la página Acerca de
use pages::asignaciones::{
    auditar_bitacora, crear_asignacion, devolver_asignacion, pagina_asignaciones,
};
use pages::index::pagina_index; // Importamos página principal (index)
use pages::inventario::{agregar_equipamiento, pagina_inventario};
use pages::login::{logout, pagina_login, procesar_login}; // Módulo de autenticación
use pages::soldados::{agregar_soldado, pagina_soldados}; // Módulo CRUD para soldados
use std::sync::Arc;
use tower_http::services::ServeDir;

use axum::http::header;
use axum::response::{IntoResponse, Response};
use maud::Markup;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<turso::Database>,
    pub session_key: axum_extra::extract::cookie::Key,
}

impl axum::extract::FromRef<AppState> for axum_extra::extract::cookie::Key {
    fn from_ref(state: &AppState) -> Self {
        state.session_key.clone()
    }
}

pub struct HtmlTemplate(pub Markup);

impl IntoResponse for HtmlTemplate {
    fn into_response(self) -> Response {
        let mut response = self.0.into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("text/html"),
        );
        response
    }
}

fn load_db_key() -> String {
    match std::env::var("ARMADILLOS_DB_KEY") {
        Ok(key) => {
            if key.len() != 64 || !key.chars().all(|c| c.is_ascii_hexdigit()) {
                panic!(
                    "ERROR: La clave de base de datos en ARMADILLOS_DB_KEY debe ser una cadena hexadecimal de 64 caracteres."
                );
            }
            key
        }
        Err(_) => {
            if cfg!(debug_assertions) {
                println!(
                    "⚠️  WARNING: ARMADILLOS_DB_KEY no está configurada. Usando clave de pruebas por defecto (OOTB)."
                );
                "b1bbfda4f589dc9daaf004fe21111e00dc00c98237102f5c7002a5669fc76327".to_string()
            } else {
                panic!(
                    "ERROR CRÍTICO: ARMADILLOS_DB_KEY no está configurada. En producción es obligatorio definir una clave de cifrado segura."
                );
            }
        }
    }
}

fn load_session_key() -> axum_extra::extract::cookie::Key {
    use axum_extra::extract::cookie::Key;
    match std::env::var("SESSION_KEY") {
        Ok(val) => {
            if let Some(key) = hex::decode(val.trim())
                .ok()
                .and_then(|bytes| Key::try_from(&bytes[..]).ok())
            {
                tracing::info!("Clave de sesión cargada exitosamente desde SESSION_KEY.");
                return key;
            }
            tracing::warn!(
                "La variable SESSION_KEY no contiene una clave válida en hex de 64 bytes. Generando una temporal..."
            );
            Key::generate()
        }
        Err(_) => {
            tracing::warn!(
                "Variable SESSION_KEY no encontrada. Generando una clave temporal de sesión para desarrollo."
            );
            Key::generate()
        }
    }
}

#[tokio::main]
async fn main() {
    // Inicializar logs para ver qué pasa internamente
    if cfg!(debug_assertions) {
        // En desarrollo: formato limpio y amigable para lectura humana
        tracing_subscriber::fmt::init();
    } else {
        // En producción: formato JSON estructurado para systemd-journald
        tracing_subscriber::fmt().json().init();
    }

    // Obtener la clave de cifrado
    let hexkey = load_db_key();
    let encryption_opts = turso::EncryptionOpts {
        hexkey,
        cipher: "aegis256".to_string(),
    };

    // Inicialización de TursoDB con cifrado nativo Aegis256
    let database = turso::Builder::new_local("armadillos.db")
        .experimental_encryption(true)
        .with_encryption(encryption_opts)
        .build()
        .await
        .expect("Error al inicializar TursoDB cifrada");

    // Conectar y ejecutar las migraciones de esquema y datos semilla
    let conn = database
        .connect()
        .expect("Error al conectar a la base de datos");
    db::database::ejecutar_migraciones(&conn)
        .await
        .expect("Error crítico al ejecutar migraciones de base de datos");

    // Obtener la clave de sesión
    let session_key = load_session_key();

    // Empaquetar la BD en el estado de la aplicación
    let state = AppState {
        db: Arc::new(database),
        session_key,
    };

    let app = crear_app(state);

    // Correr nuestra app, escuchando globalmente en el puerto 3000
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("🚀 Servidor inicializado con éxito");
    println!("📡 Escuchando en: http://{}", addr);
    println!("💡 Usa Ctrl+C para detenerlo");

    axum::serve(listener, app).await.unwrap();
}

use tower_http::set_header::SetResponseHeaderLayer;

/// Construye e inicializa el enrutador de Axum de la aplicación.
pub fn crear_app(state: AppState) -> Router {
    let routes = Router::new()
        .route("/", get(pagina_index))
        .route("/about", get(pagina_about))
        .route("/soldados", get(pagina_soldados).post(agregar_soldado))
        .route(
            "/inventario",
            get(pagina_inventario).post(agregar_equipamiento),
        )
        .route(
            "/asignaciones",
            get(pagina_asignaciones).post(crear_asignacion),
        )
        .route(
            "/asignaciones/devolver",
            axum::routing::post(devolver_asignacion),
        )
        .route("/asignaciones/auditoria", get(auditar_bitacora))
        .route("/login", get(pagina_login).post(procesar_login))
        .route("/logout", axum::routing::post(logout))
        .layer(axum::middleware::from_fn(security::csrf_middleware));

    Router::new()
        .merge(routes)
        .nest_service("/assets", ServeDir::new("assets"))
        // --- MIDDLEWARES GLOBALES DE SEGURIDAD (VULN-05 Hardening) ---
        // CSP Estricto: Sólo permitir recursos propios e inline styles genéricos
        .layer(SetResponseHeaderLayer::overriding(
            header::CONTENT_SECURITY_POLICY,
            header::HeaderValue::from_static(
                "default-src 'self'; style-src 'self' 'unsafe-inline'",
            ),
        ))
        // HSTS (HTTP Strict Transport Security): Forzar HTTPS por 2 años, cubriendo subdominios
        .layer(SetResponseHeaderLayer::overriding(
            header::STRICT_TRANSPORT_SECURITY,
            header::HeaderValue::from_static("max-age=63072000; includeSubDomains"),
        ))
        // X-Frame-Options: Bloquear que Armadillos sea embebido en un iFrame en otros dominios (prevención Clickjacking)
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            header::HeaderValue::from_static("DENY"),
        ))
        .with_state(state)
}

#[cfg(test)]
mod tests_integracion {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt;

    async fn configurar_db_pruebas(nombre_test: &str) -> AppState {
        let hexkey = "b1bbfda4f589dc9daaf004fe21111e00dc00c98237102f5c7002a5669fc76327".to_string();
        let encryption_opts = turso::EncryptionOpts {
            hexkey,
            cipher: "aegis256".to_string(),
        };

        let db_path = format!("{}.db", nombre_test);
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_file(format!("{}-journal", db_path));
        let _ = std::fs::remove_file(format!("{}-wal", db_path));

        let database = turso::Builder::new_local(&db_path)
            .experimental_encryption(true)
            .with_encryption(encryption_opts)
            .build()
            .await
            .unwrap();

        let conn = database.connect().unwrap();
        crate::db::database::ejecutar_migraciones(&conn)
            .await
            .unwrap();

        // Insertar asignación semilla de prueba (ID 1: asignación de 2 fusiles FX-05 al soldado 3, autorizado por 1)
        // Calculamos el hash-chain con hash génesis para consistencia
        let fecha_test = "2026-01-01 00:00:00";
        let hash_semilla = crate::security::calcular_hash_evento(
            "ASIGNACION",
            3,
            1,
            2,
            fecha_test,
            1,
            None,
            crate::security::HASH_GENESIS,
        );
        conn.execute(
            "INSERT INTO asignaciones_equipamiento (id, tipo_evento, soldado_id, equipamiento_id, cantidad, fecha_evento, autorizado_por_soldado_id, hash_verificacion) VALUES
             (1, 'ASIGNACION', 3, 1, 2, ?1, 1, ?2)",
            (fecha_test, hash_semilla),
        )
        .await
        .unwrap();

        // Ajustar el stock disponible de forma consistente
        conn.execute(
            "UPDATE equipamiento SET stock_disponible = stock_disponible - 2 WHERE id = 1",
            (),
        )
        .await
        .unwrap();

        AppState {
            db: Arc::new(database),
            session_key: axum_extra::extract::cookie::Key::generate(),
        }
    }

    fn limpiar_db_pruebas(nombre_test: &str) {
        let db_path = format!("{}.db", nombre_test);
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_file(format!("{}-journal", db_path));
        let _ = std::fs::remove_file(format!("{}-wal", db_path));
    }

    fn crear_cookie_encriptada(state: &AppState, name: &str, value: &str) -> String {
        use cookie::{Cookie, CookieJar};
        let mut jar = CookieJar::new();
        jar.private_mut(&state.session_key)
            .add(Cookie::new(name.to_string(), value.to_string()));
        let cookie = jar.get(name).unwrap();
        cookie.to_string()
    }

    #[tokio::test]
    async fn test_get_inventario_redirige_a_login() {
        let nombre_test = "test_get_inv_redir";
        let state = configurar_db_pruebas(nombre_test).await;
        let app = crear_app(state);

        let req = Request::builder()
            .uri("/inventario")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response
                .headers()
                .get("location")
                .unwrap()
                .to_str()
                .unwrap(),
            "/login"
        );
        limpiar_db_pruebas(nombre_test);
    }

    #[tokio::test]
    async fn test_get_inventario_con_sesion_ok() {
        let nombre_test = "test_get_inv_ok";
        let state = configurar_db_pruebas(nombre_test).await;
        let app = crear_app(state.clone());

        let cookie_str = crear_cookie_encriptada(&state, "operador_soldado_id", "3");
        let req = Request::builder()
            .uri("/inventario")
            .header("Cookie", cookie_str)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        limpiar_db_pruebas(nombre_test);
    }

    #[tokio::test]
    async fn test_post_inventario_bloqueado_si_no_es_operador_autorizado() {
        let nombre_test = "test_post_inv_bloqueado";
        let state = configurar_db_pruebas(nombre_test).await;
        let app = crear_app(state.clone());

        // Intentamos agregar un Fusil FX-05 (categoria_id = 1) con operador_soldado_id = 3 (Cabo Infantería)
        let form_data = "codigo_inventario=ARM-999&nombre=Fusil+Test&categoria_id=1&estado_conservacion=Operativo&stock_total=10";

        let cookie_str = crear_cookie_encriptada(&state, "operador_soldado_id", "3");
        let token = "testtesttesttesttesttesttesttesttesttesttesttesttesttesttesttest";
        let req = Request::builder()
            .method("POST")
            .uri("/inventario")
            .header("Cookie", format!("{}; __Host-csrf={}", cookie_str, token))
            .header("X-CSRF-Token", token)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from(form_data))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        limpiar_db_pruebas(nombre_test);
    }

    #[tokio::test]
    async fn test_post_inventario_autorizado_para_material_no_belico_incluso_tropa() {
        let nombre_test = "test_post_inv_no_belico";
        let state = configurar_db_pruebas(nombre_test).await;
        let app = crear_app(state.clone());

        // Intentamos agregar un "Uniforme" (categoria_id = 3, Equipo Táctico Individual, NO bélico) con operador_soldado_id = 3 (Cabo Infantería)
        let form_data = "codigo_inventario=TAC-999&nombre=Uniforme+Test&categoria_id=3&estado_conservacion=Operativo&stock_total=10";

        let cookie_str = crear_cookie_encriptada(&state, "operador_soldado_id", "3");
        let token = "testtesttesttesttesttesttesttesttesttesttesttesttesttesttesttest";
        let req = Request::builder()
            .method("POST")
            .uri("/inventario")
            .header("Cookie", format!("{}; __Host-csrf={}", cookie_str, token))
            .header("X-CSRF-Token", token)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from(form_data))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        limpiar_db_pruebas(nombre_test);
    }

    #[tokio::test]
    async fn test_post_inventario_autorizado_para_material_belico_si_es_oficial() {
        let nombre_test = "test_post_inv_belico_oficial";
        let state = configurar_db_pruebas(nombre_test).await;
        let app = crear_app(state.clone());

        // Intentamos agregar un Fusil (categoria_id = 1) con operador_soldado_id = 1 (Subteniente Santiago Mendoza, Oficial)
        let form_data = "codigo_inventario=ARM-888&nombre=Fusil+Oficial&categoria_id=1&estado_conservacion=Operativo&stock_total=5";

        let cookie_str = crear_cookie_encriptada(&state, "operador_soldado_id", "1");
        let token = "testtesttesttesttesttesttesttesttesttesttesttesttesttesttesttest";
        let req = Request::builder()
            .method("POST")
            .uri("/inventario")
            .header("Cookie", format!("{}; __Host-csrf={}", cookie_str, token))
            .header("X-CSRF-Token", token)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from(form_data))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        limpiar_db_pruebas(nombre_test);
    }

    #[tokio::test]
    async fn test_asignacion_material_belico_bloqueado_si_no_es_autorizante_valido() {
        let nombre_test = "test_asig_belico_bloqueado";
        let state = configurar_db_pruebas(nombre_test).await;
        let app = crear_app(state.clone());

        // Intentamos asignar un Fusil FX-05 (equipamiento_id = 1) con operador simulado Cabo (id = 3)
        let form_data = "soldado_id=1&equipamiento_id=1&cantidad=1";

        let cookie_str = crear_cookie_encriptada(&state, "operador_soldado_id", "3");
        let token = "testtesttesttesttesttesttesttesttesttesttesttesttesttesttesttest";
        let req = Request::builder()
            .method("POST")
            .uri("/asignaciones")
            .header("Cookie", format!("{}; __Host-csrf={}", cookie_str, token))
            .header("X-CSRF-Token", token)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from(form_data))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        limpiar_db_pruebas(nombre_test);
    }

    #[tokio::test]
    async fn test_devolucion_material_belico_por_oficial_exito() {
        let nombre_test = "test_dev_exito";
        let state = configurar_db_pruebas(nombre_test).await;

        // Operador Oficial (id = 1), intenta devolver la asignación semilla id = 1 (2 fusiles del equipo 1)
        let form_data = "asignacion_id=1";

        let cookie_str = crear_cookie_encriptada(&state, "operador_soldado_id", "1");
        let token = "testtesttesttesttesttesttesttesttesttesttesttesttesttesttesttest";
        let req = Request::builder()
            .method("POST")
            .uri("/asignaciones/devolver")
            .header("Cookie", format!("{}; __Host-csrf={}", cookie_str, token))
            .header("X-CSRF-Token", token)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from(form_data))
            .unwrap();

        let app = crear_app(state.clone());
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Verificar en BD que el stock disponible volvió a 150 (estaba en 148 tras la asignación semilla)
        let conn = state.db.connect().unwrap();
        let mut rows = conn
            .query("SELECT stock_disponible FROM equipamiento WHERE id = 1", ())
            .await
            .unwrap();
        let row = rows.next().await.unwrap().unwrap();
        let stock: i64 = row.get(0).unwrap();
        assert_eq!(stock, 150);

        limpiar_db_pruebas(nombre_test);
    }

    #[tokio::test]
    async fn test_devolucion_material_belico_por_tropa_bloqueado() {
        let nombre_test = "test_dev_bloqueado";
        let state = configurar_db_pruebas(nombre_test).await;

        // Operador Tropa (id = 3, Cabo Infantería), intenta devolver la asignación semilla id = 1
        let form_data = "asignacion_id=1";

        let cookie_str = crear_cookie_encriptada(&state, "operador_soldado_id", "3");
        let token = "testtesttesttesttesttesttesttesttesttesttesttesttesttesttesttest";
        let req = Request::builder()
            .method("POST")
            .uri("/asignaciones/devolver")
            .header("Cookie", format!("{}; __Host-csrf={}", cookie_str, token))
            .header("X-CSRF-Token", token)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from(form_data))
            .unwrap();

        let app = crear_app(state.clone());
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // Verificar que el stock disponible NO cambió (sigue en 148)
        let conn = state.db.connect().unwrap();
        let mut rows = conn
            .query("SELECT stock_disponible FROM equipamiento WHERE id = 1", ())
            .await
            .unwrap();
        let row = rows.next().await.unwrap().unwrap();
        let stock: i64 = row.get(0).unwrap();
        assert_eq!(stock, 148);

        limpiar_db_pruebas(nombre_test);
    }
}
