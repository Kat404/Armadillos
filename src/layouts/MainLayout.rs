use maud::{Markup, html};

pub struct PageContext<'a> {
    pub head_title: &'a str,
    pub body_title: &'a str,
    pub description: &'a str,
}

pub fn layout(ctx: &PageContext, contenido: Markup) -> Markup {
    html! {
        (maud::DOCTYPE)
        html {
            head {
                title { (ctx.head_title) }
                meta name="description" content=(ctx.description);
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                meta name="robots" content="index, follow";
                link rel="stylesheet" href="/styles/pico.min.css";
            }
            body {
                header {
                    h1 {
                        (ctx.body_title)
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
