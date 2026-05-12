# 🏗️ Flujo de Trabajo y Arquitectura: Armadillos

Esta guía establece los estándares arquitectónicos para escalar el proyecto Armadillos usando Rust, Axum y Maud. Sigue estos lineamientos al crear nuevas vistas, componentes o integrar archivos estáticos.

## 1. Gestión de Archivos Estáticos (CSS, Imágenes)

Los archivos estáticos **nunca** deben servirse desde el directorio fuente (`src/`). Esto es una vulnerabilidad de seguridad y mala práctica arquitectónica.

### Estándar de la Industria

- **Ubicación:** Todos los archivos estáticos deben residir en la carpeta raíz `assets/`.
- **Estructura:** Organiza por tipo. Por ejemplo, el CSS debe ir en `assets/css/pico.min.css`.
- **Invocación en Maud:** Llama al recurso usando rutas absolutas relativas al endpoint servido por Axum:

  ```rust
  link rel="stylesheet" href="/assets/css/pico.min.css";
  ```

## 2. Componentes Reutilizables (Modularización UI)

En Maud, un componente es una función pura que retorna un tipo `Markup`.

### Flujo de Creación

1. **Directorio:** Usa la carpeta `src/components/`.
2. **Archivo:** Crea un archivo para el componente, ej. `src/components/navbar.rs`.
3. **Módulo:** Declara el archivo en `src/components/mod.rs` (`pub mod navbar;`).
4. **Función:** Define una función pública que devuelva `Markup`:

   ```rust
   use maud::{html, Markup};

   pub fn render() -> Markup {
       html! {
           nav {
               // ... contenido del nav
           }
       }
   }
   ```

5. **Uso:** Importa e invoca la función en tu layout o página: `(navbar::render())`.

## 3. Escalabilidad de Layouts (Master Pages)

Los layouts son funciones envoltorio que estructuran páginas completas, inyectando dependencias (CSS, scripts) y metadatos compartidos.

### Flujo de Creación

1. **Directorio:** Usa la carpeta `src/layouts/`.
2. **Archivo:** Crea el layout, ej. `src/layouts/AdminLayout.rs`.
3. **Módulo:** Exponlo en `src/layouts/mod.rs` (`pub mod AdminLayout;`).
4. **Contexto y Función:** Recibe el contexto necesario y el contenido interno (`Markup`).
   ```rust
   pub fn layout(ctx: &PageContext, contenido: Markup) -> Markup {
       html! {
           // ... estructura html con (contenido) inyectado
       }
   }
   ```

## 4. Ciclo de Vida: Añadiendo Nuevas Páginas y Rutas

El flujo unidireccional para añadir una nueva vista:

### Paso A: Creación de la Vista (Página)

1. Crea un archivo en `src/pages/` (ej. `About.rs`).
2. Decláralo en `src/pages/mod.rs` (`pub mod About;`).
3. Define un "handler" asíncrono que retorne tu estructura `HtmlTemplate` (la cual asegura el Content-Type `text/html`).

   ```rust
   pub async fn pagina_about() -> HtmlTemplate {
       let ctx = PageContext { /* ... */ };
       HtmlTemplate(MainLayout::layout(&ctx, html! {
           // ... contenido específico de la página
       }))
   }
   ```

### Paso B: Mapeo en el Enrutador (Axum)

1. En `src/main.rs`, importa el nuevo handler.
2. Añade la ruta en el `Router`:

   ```rust
   let app = Router::new()
       // ... otras rutas
       .route("/about", get(pagina_about));
   ```

---

**Resumen del Mindset:**

- **Assets:** Estáticos en `/assets`.
- **Componentes:** Retornan `Markup` (`/components`).
- **Layouts:** Envoltorios de `Markup` (`/layouts`).
- **Páginas:** Handlers asíncronos que unen Contexto + Layout + Contenido (`/pages`).
- **Router:** Índice maestro en `main.rs`.
