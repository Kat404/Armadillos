# Guía de Desarrollo: Integración de Turso DB (Local-First)

Esta guía documenta los aprendizajes, decisiones de arquitectura y la implementación técnica del motor de base de datos local Turso DB (antes Limbo) en el proyecto Armadillos.

## Contexto y Decisiones de Arquitectura

Durante la fase experimental del proyecto, decidimos implementar Turso DB en su modalidad local-first. Esta decisión se tomó evaluando los siguientes aspectos:

- **Seguridad en Memoria (Rust-Native):** A diferencia de SQLite tradicional, que está escrito en C y requiere llamadas inseguras FFI (`unsafe`), el nuevo motor de Turso está escrito enteramente en Rust.
- **Asincronía Nativa:** Soporta `async/await` de forma nativa sin bloquear el hilo principal de ejecución de Tokio, lo que evita delegar tareas a pools de bloqueo externos.
- **Privacidad Absoluta:** No se utilizan las características de sincronización en la nube (Turso Cloud), confinando la base de datos exclusivamente al archivo local `armadillos.db` en el disco.

---

## Estructura de la Base de Datos (Esquema 3FN)

El esquema se inicializa automáticamente al arrancar la aplicación mediante el módulo `src/db/migrations.rs`, que ejecuta todas las sentencias DDL y datos semilla como una transacción atómica vía `execute_batch`. Las tablas están diseñadas en **Tercera Forma Normal (3FN)** para eliminar redundancias y dependencias transitivas.

### Tablas de Lookup (Catálogos)

#### rangos

Catálogo oficial de rangos militares mexicanos (SEDENA/SEMAR).

| Campo              | Tipo    | Restricción               | Descripción                                                     |
| :----------------- | :------ | :------------------------ | :-------------------------------------------------------------- |
| `id`               | INTEGER | PRIMARY KEY AUTOINCREMENT | Identificador único                                             |
| `nombre`           | TEXT    | NOT NULL UNIQUE           | Nombre del rango (ej: "Coronel")                                |
| `categoria`        | TEXT    | NOT NULL                  | Categoría orgánica (Tropa, Clases, Oficiales, Jefes, Generales) |
| `orden_jerarquico` | INTEGER | NOT NULL UNIQUE           | Nivel numérico para comparaciones de acceso ABAC                |

#### secciones_servicios

Catálogo de secciones/servicios orgánicos de un plantel militar.

| Campo                | Tipo    | Restricción               | Descripción                                                             |
| :------------------- | :------ | :------------------------ | :---------------------------------------------------------------------- |
| `id`                 | INTEGER | PRIMARY KEY AUTOINCREMENT | Identificador único                                                     |
| `nombre`             | TEXT    | NOT NULL UNIQUE           | Nombre de la sección (ej: "Materiales de Guerra")                       |
| `es_servicio_belico` | INTEGER | NOT NULL DEFAULT 0        | Flag ABAC: 1 si el personal de clases tiene acceso al inventario bélico |

#### categorias_equipamiento

Clasificación de tipos de equipamiento con flag de control de acceso.

| Campo                   | Tipo    | Restricción               | Descripción                                            |
| :---------------------- | :------ | :------------------------ | :----------------------------------------------------- |
| `id`                    | INTEGER | PRIMARY KEY AUTOINCREMENT | Identificador único                                    |
| `nombre`                | TEXT    | NOT NULL UNIQUE           | Nombre de la categoría (ej: "Armamento", "Municiones") |
| `es_material_de_guerra` | INTEGER | NOT NULL DEFAULT 0        | Flag: 1 si requiere permisos ABAC para CRUD            |

### Tablas Operativas

#### soldados

Registro de personal militar con claves foráneas a las tablas de lookup.

| Campo                 | Tipo    | Restricción                           | Descripción                                     |
| :-------------------- | :------ | :------------------------------------ | :---------------------------------------------- |
| `id`                  | INTEGER | PRIMARY KEY AUTOINCREMENT             | Identificador único interno                     |
| `matricula`           | TEXT    | NOT NULL UNIQUE                       | Clave natural de negocio (matrícula militar)    |
| `nombre`              | TEXT    | NOT NULL                              | Nombre(s) de pila                               |
| `apellido_paterno`    | TEXT    | NOT NULL                              | Apellido paterno                                |
| `apellido_materno`    | TEXT    | (nullable)                            | Apellido materno (opcional)                     |
| `rango_id`            | INTEGER | NOT NULL FK → rangos(id)              | Referencia al rango asignado                    |
| `seccion_servicio_id` | INTEGER | NOT NULL FK → secciones_servicios(id) | Referencia a la sección de servicio             |
| `estado`              | TEXT    | NOT NULL DEFAULT 'Activo'             | Estado de servicio (Activo, Licencia, Retirado) |

#### equipamiento

Inventario físico del plantel con código de inventario único.

| Campo                 | Tipo    | Restricción                               | Descripción                                   |
| :-------------------- | :------ | :---------------------------------------- | :-------------------------------------------- |
| `id`                  | INTEGER | PRIMARY KEY AUTOINCREMENT                 | Identificador único                           |
| `codigo_inventario`   | TEXT    | NOT NULL UNIQUE                           | Código de trazabilidad física (ej: "ARM-001") |
| `nombre`              | TEXT    | NOT NULL                                  | Nombre del ítem                               |
| `descripcion`         | TEXT    | (nullable)                                | Descripción técnica del equipo                |
| `categoria_id`        | INTEGER | NOT NULL FK → categorias_equipamiento(id) | Referencia a la categoría                     |
| `estado_conservacion` | TEXT    | NOT NULL DEFAULT 'Operativo'              | Estado físico del equipo                      |
| `stock_total`         | INTEGER | NOT NULL DEFAULT 0                        | Cantidad total en inventario                  |
| `stock_disponible`    | INTEGER | NOT NULL DEFAULT 0                        | Cantidad disponible (no asignada)             |

#### asignaciones_equipamiento

Cadena de custodia: registro de quién recibió qué equipo y quién lo autorizó.

| Campo                       | Tipo    | Restricción                      | Descripción                           |
| :-------------------------- | :------ | :------------------------------- | :------------------------------------ |
| `id`                        | INTEGER | PRIMARY KEY AUTOINCREMENT        | Identificador único                   |
| `soldado_id`                | INTEGER | NOT NULL FK → soldados(id)       | Soldado que recibe el equipo          |
| `equipamiento_id`           | INTEGER | NOT NULL FK → equipamiento(id)   | Equipo asignado                       |
| `cantidad`                  | INTEGER | NOT NULL DEFAULT 1               | Cantidad asignada                     |
| `fecha_asignacion`          | TEXT    | NOT NULL DEFAULT datetime('now') | Timestamp de la asignación            |
| `fecha_devolucion`          | TEXT    | (nullable)                       | NULL mientras el equipo esté asignado |
| `autorizado_por_soldado_id` | INTEGER | NOT NULL FK → soldados(id)       | Soldado que autorizó la entrega       |

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
