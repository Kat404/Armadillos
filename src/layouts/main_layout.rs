// Importamos Maud y nuestro componentes para conformar en su totalidad nuestro Layout
use crate::components::{footer, header};
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
            link rel="stylesheet" href="/assets/beercss/beer.min.css";
            link rel="stylesheet" href="/assets/css/style.css";
            script type="module" src="/assets/beercss/beer.min.js" {}
            script type="module" src="https://cdn.jsdelivr.net/npm/material-dynamic-colors@1.1.4/dist/cdn/material-dynamic-colors.min.js" {}
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
                (header::header(ctx.body_title))
                main class="responsive" {
                    (contenido)
                }
                (footer::footer())

                script type="module" {
                    (maud::PreEscaped(r#"
                        import ui from '/assets/beercss/beer.min.js';
                        
                        // Inicialización de colores dinámicos con el color base militar
                        ui('theme', '#355e3b');
                        
                        // Inicialización y persistencia del modo (claro/oscuro)
                        let currentMode = localStorage.getItem('theme-mode') || 
                            (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
                        ui('mode', currentMode);
                        
                        // Configurar el botón alternador de tema
                        const toggleBtn = document.getElementById('theme-toggle');
                        if (toggleBtn) {
                            const icon = toggleBtn.querySelector('i');
                            if (icon) {
                                icon.textContent = currentMode === 'dark' ? 'light_mode' : 'dark_mode';
                            }
                            toggleBtn.addEventListener('click', () => {
                                currentMode = currentMode === 'dark' ? 'light' : 'dark';
                                ui('mode', currentMode);
                                localStorage.setItem('theme-mode', currentMode);
                                if (icon) {
                                    icon.textContent = currentMode === 'dark' ? 'light_mode' : 'dark_mode';
                                }
                            });
                        }
                    "#))
                }
            }
        }
    }
}
