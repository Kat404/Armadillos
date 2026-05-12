// Importamos Maud y nuestro componentes para conformar en su totalidad nuestro Layout
use crate::components::{Footer, Header, NavBar};
use maud::{DOCTYPE, Markup, html};

pub struct PageContext<'a> {
    pub head_title: &'a str,
    pub body_title: &'a str,
    pub description: &'a str,
}

impl<'a> PageContext<'a> {
    pub fn new(head_title: &'a str, body_title: &'a str, description: &'a str) -> Self {
        Self {
            head_title,
            body_title,
            description,
        }
    }
}

// Función para declarar el <head> de la página
// <head> gestiona todos los metadatos, assets, vínculos CSS y mucho más
fn head(ctx: &PageContext) -> Markup {
    html! {
        head {
            title { (ctx.head_title) }
            meta name="description" content=(ctx.description);
            meta charset="UTF-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            meta name="robots" content="index, follow";
            link rel="stylesheet" href="/assets/css/pico.min.css";
        }
    }
}

// Layout 0_X
pub fn layout(ctx: &PageContext, contenido: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            (head(ctx))
            body {
                (Header::header(ctx.body_title))
                (NavBar::navbar())
                main {
                    (contenido)
                }
                (Footer::footer())
            }
        }
    }
}
