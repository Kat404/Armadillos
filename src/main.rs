mod components; // Declaramos módulo de componentes/
mod domain; // Módulo para tipos seguros de dominio militar
mod layouts; // Declaramos módulo raíz de los layouts/
mod pages; // Declaramos módulo raíz de las pages/
mod security; // Módulo de seguridad (CSRF)
use axum::{Router, routing::get};
use pages::about::pagina_about; // Importamos la página Acerca de
use pages::index::pagina_index; // Importamos página principal (index)
use pages::soldiers::{agregar_soldado, pagina_soldiers}; // Módulo CRUD para soldados
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
    let db = turso::Builder::new_local("armadillos.db")
        .experimental_encryption(true)
        .with_encryption(encryption_opts)
        .build()
        .await
        .expect("Error al inicializar TursoDB cifrada");

    // Conectar y crear la tabla 'soldados' si no existe
    let conn = db.connect().expect("Error al conectar a la base de datos");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS soldados(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            rango TEXT NOT NULL,
            estado TEXT NOT NULL
        )",
        (),
    )
    .await
    .expect("Error al crear la tabla 'soldados'");

    // Empaquetar la BD en el estado de la aplicación
    let state = AppState { db: Arc::new(db) };

    // Construir la app con una sola ruta simple y logs de peticiones
    let routes = Router::new()
        .route("/", get(pagina_index))
        .route("/about", get(pagina_about))
        .route("/soldiers", get(pagina_soldiers).post(agregar_soldado))
        .layer(axum::middleware::from_fn(security::csrf_middleware));

    let app = Router::new()
        .merge(routes)
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state);

    // Correr nuestra app, escuchando globalmente en el puerto 3000
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("🚀 Servidor inicializado con éxito");
    println!("📡 Escuchando en: http://{}", addr);
    println!("💡 Usa Ctrl+C para detenerlo");

    axum::serve(listener, app).await.unwrap();
}
