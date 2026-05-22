mod components; // Declaramos módulo de componentes/
mod layouts; // Declaramos módulo raíz de los layouts/
mod pages; // Declaramos módulo raíz de las pages/
use axum::{Router, routing::get};
use pages::About::pagina_about; // Importamos la página Acerca de
use pages::Main::pagina_main; // Importamos página principal (index)
use tower_http::services::ServeDir;

use axum::http::header;
use axum::response::{IntoResponse, Response};
use maud::Markup;

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

    // Construir la app con una sola ruta simple y logs de peticiones
    let app = Router::new()
        .route("/", get(pagina_main))
        .route("/about", get(pagina_about))
        .nest_service("/assets", ServeDir::new("src/assets"));

    // Correr nuestra app, escuchando globalmente en el puerto 3000
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("🚀 Servidor inicializado con éxito");
    println!("📡 Escuchando en: http://{}", addr);
    println!("💡 Usa Ctrl+C para detenerlo");

    axum::serve(listener, app).await.unwrap();
}
