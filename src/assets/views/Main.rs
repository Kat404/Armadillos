use crate::assets::layouts::MainLayout::layout;
use maud::{Markup, html};

pub async fn pagina_main() -> Markup {
    layout(
        "Bienvenido",
        html! {
            h2 {
                "¡Bienvenido a esta página web desarrollada 100% en Rust!"
            }
            p {
                "Es sumamente veloz, eficiente en memoria y sobresaliente en seguridad."
            }
        },
    )
}
