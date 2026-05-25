use crate::layouts::main_layout::{PageContext, layout}; // Importamos nuestro Layout
use maud::{Markup, html};

pub async fn pagina_index() -> Markup {
    // Definimos el contexto de nuestra página
    let ctx = PageContext::new(
        "Armadillos - Alternativa FOSS, privada y segura para administración militar",
        "¡Bienvenido a Armadillos!",
        "Armadillos - Software seguro y privado mediante un uso e implementación 100% en Rust, con tipado fuerte, seguridad de memoria y un rendimiento excelente",
    );

    layout(
        &ctx,
        html! {
            h2 {
                "¿Estás listo para un software de nueva generación?"
            }
            p {
                "Es sumamente veloz, eficiente en memoria y sobresaliente en seguridad."
            }
        },
    )
}
