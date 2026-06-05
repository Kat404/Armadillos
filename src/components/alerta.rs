use maud::{Markup, html};

/// Genera un banner de alerta dinámico con estilos oficiales de BeerCSS.
pub fn alerta(tipo: &str, titulo: &str, mensaje: &str) -> Markup {
    let (icon, container_class) = match tipo {
        "success" => ("check_circle", "success-container success-border"),
        "warning" => ("warning", "warning-container warning-border"),
        _ => ("error", "error-container error-border"),
    };

    html! {
        div class={"row align-center border round padding margin-bottom " (container_class)} id="alerta-dinamica" {
            i { (icon) }
            div class="max" {
                h6 class="bold no-margin" { (titulo) }
                p class="caption no-margin text-secondary" { (mensaje) }
            }
            button class="circle transparent" onclick="document.getElementById('alerta-dinamica').remove()" {
                i { "close" }
            }
        }
    }
}
