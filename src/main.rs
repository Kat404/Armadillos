mod components; // Declaramos módulo de componentes/
mod db; // Módulo de base de datos (migraciones y esquema)
mod domain; // Módulo para tipos seguros de dominio militar
mod layouts; // Declaramos módulo raíz de los layouts/
mod pages; // Declaramos módulo raíz de las pages/
mod security; // Módulo de seguridad (CSRF)
use axum::{Router, routing::get};
use pages::about::pagina_about; // Importamos la página Acerca de
use pages::asignaciones::{crear_asignacion, devolver_asignacion, pagina_asignaciones};
use pages::index::pagina_index; // Importamos página principal (index)
use pages::inventario::{agregar_equipamiento, pagina_inventario};
use pages::soldados::{agregar_soldado, pagina_soldados, simular_operador}; // Módulo CRUD para soldados
use std::sync::Arc;
use tower_http::services::ServeDir;

use axum::http::header;
use axum::response::{IntoResponse, Response};
use maud::Markup;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<turso::Database>,
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

    // Empaquetar la BD en el estado de la aplicación
    let state = AppState {
        db: Arc::new(database),
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
        .route("/simular_operador", axum::routing::post(simular_operador))
        .layer(axum::middleware::from_fn(security::csrf_middleware));

    Router::new()
        .merge(routes)
        .nest_service("/assets", ServeDir::new("assets"))
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

        // Insertar soldados de prueba para la evaluación de políticas ABAC
        conn.execute(
            "INSERT INTO soldados (id, matricula, nombre, apellido_paterno, rango_id, seccion_servicio_id, estado) VALUES
             (1, 'M-2309401', 'Santiago', 'Mendoza', 6, 1, 'Activo'),
             (2, 'M-2309402', 'Mateo',    'Guerrero', 4, 6, 'Activo'),
             (3, 'M-2309403', 'Sebastián','Ortega',   3, 2, 'Activo')",
            (),
        )
        .await
        .unwrap();

        // Insertar asignación semilla de prueba (ID 1: asignación de 2 fusiles FX-05 al soldado 3, autorizado por 1)
        conn.execute(
            "INSERT INTO asignaciones_equipamiento (id, soldado_id, equipamiento_id, cantidad, autorizado_por_soldado_id) VALUES
             (1, 3, 1, 2, 1)",
            (),
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
        }
    }

    fn limpiar_db_pruebas(nombre_test: &str) {
        let db_path = format!("{}.db", nombre_test);
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_file(format!("{}-journal", db_path));
        let _ = std::fs::remove_file(format!("{}-wal", db_path));
    }

    #[tokio::test]
    async fn test_get_inventario_redirige_a_operador_defecto() {
        let nombre_test = "test_get_inv";
        let state = configurar_db_pruebas(nombre_test).await;
        let app = crear_app(state);

        let req = Request::builder()
            .uri("/inventario")
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
        let app = crear_app(state);

        // Intentamos agregar un Fusil FX-05 (categoria_id = 1) con operador_soldado_id = 3 (Cabo Infantería)
        let form_data = "codigo_inventario=ARM-999&nombre=Fusil+Test&categoria_id=1&estado_conservacion=Operativo&stock_total=10";

        let token = "testtesttesttesttesttesttesttesttesttesttesttesttesttesttesttest";
        let req = Request::builder()
            .method("POST")
            .uri("/inventario")
            .header(
                "Cookie",
                format!("operador_soldado_id=3; __Host-csrf={}", token),
            )
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
        let app = crear_app(state);

        // Intentamos agregar un "Uniforme" (categoria_id = 3, Equipo Táctico Individual, NO bélico) con operador_soldado_id = 3 (Cabo Infantería)
        let form_data = "codigo_inventario=TAC-999&nombre=Uniforme+Test&categoria_id=3&estado_conservacion=Operativo&stock_total=10";

        let token = "testtesttesttesttesttesttesttesttesttesttesttesttesttesttesttest";
        let req = Request::builder()
            .method("POST")
            .uri("/inventario")
            .header(
                "Cookie",
                format!("operador_soldado_id=3; __Host-csrf={}", token),
            )
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
        let app = crear_app(state);

        // Intentamos agregar un Fusil (categoria_id = 1) con operador_soldado_id = 1 (Subteniente Santiago Mendoza, Oficial)
        let form_data = "codigo_inventario=ARM-888&nombre=Fusil+Oficial&categoria_id=1&estado_conservacion=Operativo&stock_total=5";

        let token = "testtesttesttesttesttesttesttesttesttesttesttesttesttesttesttest";
        let req = Request::builder()
            .method("POST")
            .uri("/inventario")
            .header(
                "Cookie",
                format!("operador_soldado_id=1; __Host-csrf={}", token),
            )
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
        let app = crear_app(state);

        // Intentamos asignar un Fusil FX-05 (equipamiento_id = 1) con operador simulado Cabo (id = 3)
        let form_data = "soldado_id=1&equipamiento_id=1&cantidad=1";

        let token = "testtesttesttesttesttesttesttesttesttesttesttesttesttesttesttest";
        let req = Request::builder()
            .method("POST")
            .uri("/asignaciones")
            .header(
                "Cookie",
                format!("operador_soldado_id=3; __Host-csrf={}", token),
            )
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

        let token = "testtesttesttesttesttesttesttesttesttesttesttesttesttesttesttest";
        let req = Request::builder()
            .method("POST")
            .uri("/asignaciones/devolver")
            .header(
                "Cookie",
                format!("operador_soldado_id=1; __Host-csrf={}", token),
            )
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

        let token = "testtesttesttesttesttesttesttesttesttesttesttesttesttesttesttest";
        let req = Request::builder()
            .method("POST")
            .uri("/asignaciones/devolver")
            .header(
                "Cookie",
                format!("operador_soldado_id=3; __Host-csrf={}", token),
            )
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
