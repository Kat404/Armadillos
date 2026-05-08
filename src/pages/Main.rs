use crate::layouts::MainLayout::{PageContext, layout}; // Importamos nuestro Layout
use maud::{Markup, html};

pub async fn pagina_main() -> Markup {
    // Definimos el contexto de nuestra página
    let ctx = PageContext {
        head_title: "Armadillos - Alternativa FOSS, privada y segura para administración militar",
        body_title: "¡Bienvenido a Armadillos!",
        description: "Armadillos - Software seguro y privado mediante un uso e implementación 100% en Rust, con tipado fuerte, seguridad de memoria y un rendimiento excelente",
    };

    layout(
        &ctx,
        html! {
            h2 {
                // Seguimos usando nuestras variables de contexto para ser consistentes
                (ctx.body_title)
            }
            p {
                "Es sumamente veloz, eficiente en memoria y sobresaliente en seguridad."
            }
        },
    )
}
