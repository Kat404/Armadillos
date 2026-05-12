use crate::layouts::MainLayout::{PageContext, layout}; // Importamos nuestro Layout
use maud::{Markup, html};

pub async fn pagina_about() -> Markup {
    let ctx = PageContext::new(
        "Armadillos - Alternativa FOSS, privada y segura para administración militar",
        "Acerca de Armadillos",
        "Armadillos - Descubre sus principales características y cómo compite contra otros software de la industria",
    );

    layout(
        &ctx,
        html! {
            h2 {
                "¿Qué es 'Armadillos'?"
            }
            p {
                "Armadillos es un software tipo "
                strong {
                    "web app"
                }
                " la cual tiene como propósito sevir como un software All-in-One
                para la administración, modificación, servicios, registros y mucho más dentro
                de un contexto de militar usando Rust como lenguaje de programación principal para garantizar
                una gran seguridad y rendimiento incomparable dentro de entornos industriales
                críticos y sumamente utilizados."
            }
            div {
                p {
                    "Sus principales características son:"
                }
                details {
                    summary {
                        "Seguridad y Rendimiento sin igual"
                    }
                    p {
                        "Armadillos al estar desarrollado puramente en Rust, el cual no tiene GC y tiene un tipado
                        sumamente fuerte, evitando errores en la memoria y proporcionando una velocidad increíble,
                        a eso sumando el uso del principio DIY (Do It Yourself) y KISS (Keep It Simple, Stupid)
                        se mantienen todas las variables y configuraciones del entorno, evitando grandes monolitos
                        de desarrollo para un mayor conocimiento de las capacidades de Rust como lenguaje
                        además del stack usado para integrarlo de manera eficiente y alcanzar los mayores
                        estándares de la industria"
                    }

                }
                details {
                    summary {
                        "Privacidad y Protección de los datos"
                    }
                    p {
                        "Aunque toda el caparazón de Armadillos sean seguro, se asegura que los datos nunca llegarán
                        a manos equivocadas, manteniendo permisos, accesos y logs restrictivos para lograr este objetivo.
                        Además de implementar los mejores algoritmos de encriptación y hashing para garantizar la seguridad
                        de las contraseñas e información sensible"
                    }
                }
            }
        },
    )
}
