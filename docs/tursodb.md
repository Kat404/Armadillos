# Guía de Desarrollo: Integración de Turso DB (Local-First)

Esta guía documenta los aprendizajes, decisiones de arquitectura y la implementación técnica del motor de base de datos local Turso DB (antes Limbo) en el proyecto Armadillos.

## Contexto y Decisiones de Arquitectura

Durante la fase experimental del proyecto, decidimos implementar Turso DB en su modalidad local-first. Esta decisión se tomó evaluando los siguientes aspectos:

- **Seguridad en Memoria (Rust-Native):** A diferencia de SQLite tradicional, que está escrito en C y requiere llamadas inseguras FFI (`unsafe`), el nuevo motor de Turso está escrito enteramente en Rust.
- **Asincronía Nativa:** Soporta `async/await` de forma nativa sin bloquear el hilo principal de ejecución de Tokio, lo que evita delegar tareas a pools de bloqueo externos.
- **Privacidad Absoluta:** No se utilizan las características de sincronización en la nube (Turso Cloud), confinando la base de datos exclusivamente al archivo local `armadillos.db` en el disco.

---

## Estructura de la Base de Datos

El esquema se inicializa automáticamente al arrancar la aplicación. La tabla principal de la base de datos posee la siguiente estructura:

### Tabla: soldados

Representa los registros de personal militar administrados en el sistema.

| Campo | Tipo | Restricción | Descripción |
| :--- | :--- | :--- | :--- |
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | Identificador único autoincremental |
| `nombre` | TEXT | NOT NULL | Nombre completo del soldado |
| `rango` | TEXT | NOT NULL | Rango militar (ej: Cabo, Sargento) |
| `estado` | TEXT | NOT NULL | Estado de servicio (ej: Activo, Licencia) |

---

## Flujo de Trabajo con Maud y HTMX

La interfaz implementa un patrón Hypermedia centrado en el servidor. El flujo de datos opera de la siguiente manera:

1. **Carga Inicial (`GET`):** El cliente solicita `/soldiers`. El handler lee los soldados de la base de datos y renderiza la página completa combinando el layout y las filas.
2. **Interactividad Asíncrona (`POST`):**
    - El formulario en el cliente intercepta el envío mediante el atributo `hx-post="/soldiers"`.
    - Axum procesa el registro e inserta el elemento en la base de datos de Turso.
    - El servidor responde únicamente con el fragmento HTML correspondiente a las filas actualizadas (`<tr>`).
    - HTMX inserta esta respuesta directamente en el contenedor del navegador (`tbody`) sin recargar la página.

---

## Buenas Prácticas y Aprendizajes Técnicos

### Errores Comunes de Sintaxis SQL

Al interactuar con bases de datos relacionales en crudo, es crucial validar que las sentencias de definición y manipulación no posean errores que provoquen pánicos (`panic!`) en el arranque de la aplicación:

- **Creación de esquemas:** La sintaxis correcta es `CREATE TABLE IF NOT EXISTS` (evitando errores comunes como `IF NO EXISTS`).
- **Inserción de datos:** La palabra clave correcta es `VALUES` en plural al realizar operaciones de inserción, no `VALUE`.

### Compartir Conexiones en Axum

El objeto de base de datos (`turso::Database`) representa el archivo en el disco y es seguro para ser compartido entre múltiples hilos de ejecución (`Send + Sync`).

Para implementarlo correctamente en Axum:

1. Se empaqueta en una estructura inteligente de tipo `Arc` (Atomic Reference Counting) dentro del estado de la aplicación.
2. Se inyecta al router mediante `.with_state(state)`.
3. Cada handler obtiene una referencia y abre una conexión local temporal mediante `state.db.connect()`, garantizando la concurrencia y la liberación oportuna de recursos.
