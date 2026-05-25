mod components; // Declaramos módulo de componentes/
mod layouts; // Declaramos módulo raíz de los layouts/
mod pages; // Declaramos módulo raíz de las pages/
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

#[tokio::main]
async fn main() {
    // Inicializar logs para ver qué pasa internamente
    // Se puede configurar el nivel con RUST_LOG=debug cargo run
    tracing_subscriber::fmt::init();

    // Inicialización de TursoDB
    // Creamos el archivos "armadillos.db" en la raíz del proyecto
    let db = turso::Builder::new_local("armadillos.db")
        .build()
        .await
        .expect("Error al inicializar TursoDB");

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
    let app = Router::new()
        .route("/", get(pagina_index))
        .route("/about", get(pagina_about))
        .route("/soldiers", get(pagina_soldiers).post(agregar_soldado))
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
