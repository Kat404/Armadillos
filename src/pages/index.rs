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
            // Tarjeta de Bienvenida Principal
            article class="medium-padding round border medium-margin" {

                div class="grid" {
                    div class="s12" {
                        h4 class="medium-margin" {
                            "¿Estás listo para un software de nueva generación?"
                        }
                        p class="medium-margin" {
                            "Armadillos está desarrollado puramente en Rust para garantizar el más alto nivel de seguridad de memoria, velocidad excepcional y total privacidad."
                        }
                        p class="medium-margin" {
                            "Es sumamente veloz, eficiente en memoria y sobresaliente en seguridad."
                        }
                        nav class="medium-margin" style="gap: 16px;" {
                            a href="/soldiers" class="button round primary" {
                                i { "shield" }
                                span { "Gestionar Personal" }
                            }
                            a href="/about" class="button round secondary" {
                                i { "info" }
                                span { "Conocer más" }
                            }
                        }
                    }
                }
            }

            h5 class="medium-margin" { "Arquitectura del Sistema" }

            // Rejilla responsiva 2x3 para los módulos del stack
            div class="grid" {
                // Rust
                div class="s12 m6" {
                    article class="round border card-rust medium-padding" {
                        div class="row" {
                            i { "terminal" }
                            h6 class="title" { "Rust" }
                        }
                        p {
                            "El lenguaje de programación principal. Garantiza seguridad de memoria y type-safety estricto para evitar estados inválidos en la administración militar."
                        }
                    }
                }

                // Turso
                div class="s12 m6" {
                    article class="round border card-turso medium-padding" {
                        div class="row" {
                            i { "database" }
                            h6 class="title" { "Turso / LibSQL" }
                        }
                        p {
                            "Almacenamiento privado y rápido. SQLite con replicación en la nube opcional y transacciones ACID que protegen el inventario militar y el personal enlistado."
                        }
                    }
                }

                // Axum
                div class="s12 m6" {
                    article class="round border card-axum medium-padding" {
                        div class="row" {
                            i { "router" }
                            h6 class="title" { "Axum + Tokio" }
                        }
                        p {
                            "El motor del backend. Proporciona enrutamiento asíncrono rápido y servicios web ultraeficientes sobre el runtime de Tokio."
                        }
                    }
                }

                // Maud
                div class="s12 m6" {
                    article class="round border card-maud medium-padding" {
                        div class="row" {
                            i { "html" }
                            h6 class="title" { "Maud Templates" }
                        }
                        p {
                            "Generación de HTML directamente en Rust. Compilación en tiempo de diseño que elimina errores sintácticos y optimiza drásticamente el renderizado."
                        }
                    }
                }

                // BeerCSS
                div class="s12 m6" {
                    article class="round border card-beercss medium-padding" {
                        div class="row" {
                            i { "palette" }
                            h6 class="title" { "BeerCSS" }
                        }
                        p {
                            "Diseño visual moderno que sigue los lineamientos de Material You de Google, ofreciendo adaptabilidad responsiva sin código JavaScript pesado."
                        }
                    }
                }

                // HTMX
                div class="s12 m6" {
                    article class="round border card-htmx medium-padding" {
                        div class="row" {
                            i { "bolt" }
                            h6 class="title" { "HTMX" }
                        }
                        p {
                            "Interactividad SPA a través del servidor. Transiciones rápidas e inserciones AJAX parciales sin recargar páginas ni sobrecargar el navegador del usuario."
                        }
                    }
                }
            }
        },
    )
}
