use maud::{Markup, html};

pub fn layout(titulo: &str, contenido: Markup) -> Markup {
    html! {
        (maud::DOCTYPE)
        html {
            head {
                title { (titulo) }
            }
            body {
                header {
                    h1 {
                        (titulo)
                    }
                    nav {
                        "Menú de Navegación"
                    }
                }
                main {
                    (contenido)
                }
                footer {
                    "Hecho en Rust con amor ^^"
                }
            }
        }
    }
}
