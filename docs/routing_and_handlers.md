# 🛤️ Rutas y Handlers: El Mapa de tu Web

En Axum, una ruta conecta una **URL** con una **Función de Rust** (llamada Handler). Lo increíble de Axum es que es _Type-Safe_: si intentas pedir un dato que no existe o es del tipo incorrecto, el código ni siquiera compilará.

## 1. Definiendo Rutas

Las rutas se definen en el `Router`. Cada ruta tiene un método HTTP (GET, POST, etc.) y un path.

```rust
let app = Router::new()
    .route("/", get(home_handler))         // GET a la raíz
    .route("/usuarios/:id", get(get_user)) // GET con parámetro variable
    .route("/contacto", post(send_email)); // POST para enviar datos
```

## 2. Los Handlers (Funciones)

Un handler es simplemente una función asíncrona. Puede recibir "Extractores" como argumentos.

### ¿Qué son los Extractores?

Son tipos de datos que le dicen a Axum: "Oye, saca este pedazo de información de la petición HTTP por mí".

- `Path<T>`: Para variables en la URL (ej: `/usuarios/42`).
- `Query<T>`: Para parámetros de búsqueda (ej: `?busqueda=rust`).
- `Json<T>`: Para el cuerpo de la petición en formato JSON.
- `State<T>`: Para acceder a la base de datos o configuración global.

**Ejemplo de un Handler Tipado:**

```rust
async fn get_user(
    Path(user_id): Path(u64), // Extrae el ID de la URL y lo convierte a número
) -> String {
    format!("Viendo el perfil del usuario {}", user_id)
}
```

## 3. Respuestas Dinámicas

Axum puede devolver muchas cosas:

- Una cadena de texto (`String` o `&'static str`).
- JSON (`Json(mi_estructura)`).
- HTML (Usando **Maud**, que veremos en la siguiente sección).
- Códigos de error (`StatusCode`).

---

## 🛠️ Buenas Prácticas

1.  **Mantén los handlers limpios:** Un handler solo debe recibir la petición y llamar a la lógica de negocio en otro módulo. No pongas 500 líneas de código dentro de un handler.
2.  **Usa Estados:** Si necesitas una conexión a la base de datos, no uses variables globales. Usa el extractor `State`.
3.  **Aprovecha el Sistema de Tipos:** Si un parámetro debe ser un número positivo, usa `u64` en el `Path`. Axum devolverá un error `400 Bad Request` automáticamente si alguien envía letras.

---

> [!IMPORTANT]
> El orden de las capas (`.layer()`) importa. Generalmente, las capas aplicadas al final del `Router` envuelven a todas las rutas anteriores.
