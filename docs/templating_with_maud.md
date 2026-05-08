# 🎨 Templating con Maud: HTML con Superpoderes

**Maud** es un motor de plantillas para Rust que no usa archivos `.html` externos. En su lugar, usa una macro llamada `html!` que convierte código Rust directamente en HTML ultra eficiente en tiempo de compilación.

## 1. ¿Por qué Maud?

- **Velocidad:** No hay que leer archivos del disco ni procesar texto en tiempo de ejecución. El HTML se "pre-calcula".
- **Seguridad:** Olvida los ataques XSS. Maud escapa automáticamente todo el texto.
- **Tipado:** Si intentas usar una variable que no existe dentro de tu HTML, Rust te dará un error de compilación.

## 2. Sintaxis Básica

La macro `html!` usa llaves `{}` para definir elementos y paréntesis `()` para insertar variables.

```rust
let nombre = "Armadillo";
let markup = html! {
    h1 { "Hola, " (nombre) "!" }
    p .clase-especial #id-unico { "Este es un párrafo con clase e ID." }
    ul {
        @for i in 1..=3 {
            li { "Elemento " (i) }
        }
    }
};
```

## 3. Integración con Axum

Gracias a la feature `axum` que tenemos en nuestro `Cargo.toml`, integrar Maud es ridículamente fácil. Solo tienes que devolver el tipo `Markup`.

**Ejemplo de integración:**

```rust
use maud::{html, Markup};

async fn home_handler() -> Markup {
    html! {
        (maud::DOCTYPE) // Añade <!DOCTYPE html>
        html {
            head {
                title { "Mi App de Armadillos" }
                link rel="stylesheet" href="/assets/style.css";
            }
            body {
                h1 { "Bienvenido al nido" }
                p { "Aquí empieza nuestra aventura con Rust." }
            }
        }
    }
}
```

## 4. Estructura de "Layout" (Página Base)

Para no repetir el `head`, `body` y el CSS en cada página, creamos una función de Layout:

```rust
fn layout(titulo: &str, contenido: Markup) -> Markup {
    html! {
        (maud::DOCTYPE)
        html {
            head {
                title { (titulo) }
                link rel="stylesheet" href="/assets/style.css";
            }
            body {
                nav { "Menú Principal" }
                main { (contenido) }
                footer { "Hecho con 🦀 y Maud" }
            }
        }
    }
}

// Uso en un handler:
async fn pagina_especifica() -> Markup {
    layout("Página 2", html! {
        h2 { "Contenido exclusivo" }
    })
}
```

---

## 🎨 CSS y Archivos Estáticos

Para que tu CSS funcione, necesitas que Axum sepa dónde están los archivos. Esto se hace con `ServeDir` de `tower-http`:

```rust
let app = Router::new()
    .route("/", get(home_handler))
    // Servir la carpeta "assets" en la URL "/assets"
    .nest_service("/assets", ServeDir::new("assets"));
```

---

> [!TIP]
> Maud usa una sintaxis parecida a CSS para clases y IDs: `.mi-clase` para class y `#mi-id` para id. ¡Es muy intuitivo!
