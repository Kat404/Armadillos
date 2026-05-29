use crate::layouts::main_layout::{PageContext, layout}; // Importamos nuestro Layout
use crate::security::CsrfToken;
use axum::Extension;
use maud::{Markup, html};

pub async fn pagina_about(Extension(csrf_token): Extension<CsrfToken>) -> Markup {
    let ctx = PageContext::new(
        "Armadillos - Alternativa FOSS, privada y segura para administración militar",
        "Acerca de Armadillos",
        "Armadillos - Descubre sus principales características y cómo compite contra otros software de la industria",
        &csrf_token.0,
    );

    layout(
        &ctx,
        html! {
            article class="medium-padding round border medium-margin" {
                div class="grid" {
                    div class="s12" {
                        h4 class="medium-margin" {
                            "¿Qué es 'Armadillos'?"
                        }
                        p class="medium-margin" {
                            "Armadillos es un software tipo "
                            strong {
                                "web app"
                            }
                            " la cual tiene como propósito servir como un software All-in-One
                            para la administración, modificación, servicios, registros y mucho más dentro
                            de un contexto militar usando Rust como lenguaje de programación principal para garantizar
                            una gran seguridad y rendimiento incomparable dentro de entornos industriales
                            críticos y sumamente utilizados."
                        }

                        h5 class="medium-margin" {
                            "Sus principales características son:"
                        }

                        div class="medium-margin" {
                            details class="primary-container round extra-margin" {
                                summary class="none" {
                                    div class="row wave padding" {
                                        i { "lock" }
                                        div class="col" {
                                            div class="title" { "Seguridad y Rendimiento sin igual" }
                                        }
                                        i { "expand_more" }
                                    }
                                }
                                div class="padding" {
                                    p {
                                        "Armadillos al estar desarrollado puramente en Rust, el cual no tiene GC y tiene un tipado
                                        sumamente fuerte, evitando errores en la memoria y proporcionando una velocidad increíble,
                                        a eso sumando el uso del principio DIY (Do It Yourself) y KISS (Keep It Simple, Stupid)
                                        se mantienen todas las variables y configuraciones del entorno, evitando grandes monolitos
                                        de desarrollo para un mayor conocimiento de las capacidades de Rust como lenguaje
                                        además del stack usado para integrarlo de manera eficiente y alcanzar los mayores
                                        estándares de la industria."
                                    }
                                }
                            }

                            details class="secondary-container round extra-margin" {
                                summary class="none" {
                                    div class="row wave padding" {
                                        i { "visibility_off" }
                                        div class="col" {
                                            div class="title" { "Privacidad y Protección de los datos" }
                                        }
                                        i { "expand_more" }
                                    }
                                }
                                div class="padding" {
                                    p {
                                        "Aunque toda la caparazón de Armadillos sea segura, se asegura que los datos nunca llegarán
                                        a manos equivocadas, manteniendo permisos, accesos y logs restrictivos para lograr este objetivo.
                                        Además de implementar los mejores algoritmos de encriptación y hashing para garantizar la seguridad
                                        de las contraseñas e información sensible."
                                    }
                                }
                            }

                            details class="tertiary-container round extra-margin" {
                                summary class="none" {
                                    div class="row wave padding" {
                                        i { "code" }
                                        div class="col" {
                                            div class="title" { "Minimalismo Open Source" }
                                        }
                                        i { "expand_more" }
                                    }
                                }
                                div class="padding" {
                                    p {
                                        "En el mundo no siempre se puede confiar en las grandes empresas, tenemos una confianza
                                        inconsciente hacia las Big Tech (Google, Microsoft, Apple, etc) en la cual les confiamos nuestros
                                        datos, perfiles, gustos y toda información relevante de nosotros."
                                        br;
                                        "Armadillos no busca que confíes solo por tener una buena arquitectura o decirte cosas bonitas.
                                        Busca que puedas y "
                                        strong { "debas" }
                                        " auditar el código, buscar errores, fallos, vulnerabilidades y atribuir esos cambios para
                                        mejorar el proyecto beneficiando a todos."
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}
