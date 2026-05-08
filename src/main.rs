mod assets; // Declaramos módulo raíz de los assets/
use assets::views::Main::pagina_main; // Importamos página principal
use axum::{Router, routing::get};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // Inicializar logs para ver qué pasa internamente
    // Se puede configurar el nivel con RUST_LOG=debug cargo run
    tracing_subscriber::fmt::init();

    // Construir la app con una sola ruta simple y logs de peticiones
    let app = Router::new()
        .route("/", get(pagina_main))
        .nest_service("/src/assets", ServeDir::new("assets"));

    // Correr nuestra app, escuchando globalmente en el puerto 3000
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("🚀 Servidor inicializado con éxito");
    println!("📡 Escuchando en: http://{}", addr);
    println!("💡 Usa Ctrl+C para detenerlo");

    axum::serve(listener, app).await.unwrap();
}
