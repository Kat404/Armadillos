# ROADMAP.md (V3: Normalización & ABAC)

## Fase 1: Cimientos y Seguridad Estática

_Objetivo: Establecer un entorno de desarrollo donde sea "difícil" cometer errores._

- [x] **Arquitectura Base:** Estructura de módulos y configuración de `Cargo.toml`.
- [x] **Enforcement de Estilo y Seguridad:** Configurar `clippy` con niveles estrictos (`deny(warnings)`), formateo automático e integración de pruebas unitarias obligatorias en [GEMINI.md](GEMINI.md).
- [x] **Gestión de Secretos:** Implementar un sistema para que las llaves de la DB y sales de encriptación nunca toquen el código (uso de variables de entorno con fallbacks controlados en desarrollo).
- [x] **Licenciamiento:** Definir formalmente la licencia **AGPLv3** en el repositorio para proteger el código en entornos SaaS.

## Fase 2: El Corazón del Dominio (Type-Safety Militar)

_Objetivo: Usar el sistema de tipos de Rust para que la lógica de negocio sea "imposible" de romper._

- [x] **Modelado de Jerarquías:** Implementar la jerarquía de mando usando _State Machines_ con Enums (`Rango`, `EstadoServicio`).
- [x] **Sistema de Permisos (ABAC):** Diseñar los Structs de `User` y `Context` para permitir Control de Acceso Basado en Atributos.
- [x] **Validación de Tipos (Dominio):** Eliminar la persistencia de strings crudos ("Stringly-Typed") mapeando e insertando tipos seguros de datos en los endpoints.
- [x] **Pruebas de Lógica de Dominio:** Implementar suite de pruebas unitarias (`cargo test`) para validar las reglas de la máquina de estados militar y de seguridad.

## Fase 3: Interfaz de Usuario Hypermedia (Maud + HTMX)

_Objetivo: Una UI rápida y privada sin el "overhead" de JavaScript._

- [x] **Pattern IntoResponse:** Newtype para manejo de respuestas limpias.
- [x] **Interactividad Hypermedia con HTMX y Maud:** Implementación inicial de flujo CRUD asíncrono para gestión de personal militar (`/soldiers`).
- [x] **Autocontención de Assets:** Descargar y servir localmente `htmx.min.js` y `material-dynamic-colors.min.js` en [main_layout.rs](src/layouts/main_layout.rs), removiendo la dependencia de CDNs.
- [x] **Componentización Crítica:** Crear `src/views/components/` (o `src/components/`) para elementos de UI que requieran validación visual (ej. indicadores de nivel de acceso).
- [x] **Seguridad en la Capa de Transporte:** Implementar protecciones CSRF mediante middlewares de Axum/Tower-HTTP, vital ya que HTMX usa peticiones AJAX.

## Fase 4: Persistencia y Cifrado en Reposo

_Objetivo: Blindar la base de datos contra accesos físicos no autorizados._

- [x] **Estrategia de Base de Datos (Segura y Local-First):** Integración de Turso DB (reescrita en Rust) en modo local-first en `armadillos.db` para evitar FFI unsafe de C, y documentación del flujo en [docs/tursodb.md](docs/tursodb.md).
- [x] **Cifrado de Base de Datos TursoDB:** Configurar el cifrado de página nativo en reposo de Turso (`Aegis256`) utilizando la clave del sistema.
- [x] **Application-Level Encryption (ALE):** Implementar cifrado para campos sensibles (Nombres de oficiales, ubicaciones de armamento) antes de que lleguen a la DB.
- [x] **Backups Cifrados:** Scripting (posiblemente en Nushell) para automatizar respaldos hacia almacenamiento local o remoto usando encriptación de llave pública.

## Fase 5: Auditabilidad y "Non-Repudiation"

_Objetivo: Que cada click sea rastreable y legalmente vinculante._

- [x] **Logs Estructurados (Tracing):** Configurar `tracing-subscriber` para generar logs en formato JSON estructurado, instrumentando handlers críticos con spans asíncronos.
- [ ] **Integridad de Logs:** Investigar e implementar un sistema de "Hashing" de logs para detectar si alguien (incluso un admin de sistema) ha alterado los registros de acceso.
- [ ] **Hardening de Producción:** Configuración de cabeceras de seguridad (HSTS, CSP) y despliegue sobre una base de Arch Linux minimalista o contenedores Alpine.

---

## Fase 6: Normalización de BD e Inventario Militar Mexicano

_Objetivo: Adaptar la base de datos al contexto real de las Fuerzas Armadas de México y establecer un inventario bélico auditable._

### Iteración 1: Migración de BD a Esquema 3FN ✅

- [x] **Módulo de Migraciones Centralizado:** Crear `src/db/database.rs` con esquema DDL completo, datos semilla y transacciones atómicas vía `execute_batch`, eliminando SQL inline de `main.rs`.
- [x] **Esquema Normalizado (3FN):** 6 tablas interconectadas por claves foráneas (`rangos`, `secciones_servicios`, `soldados`, `categorias_equipamiento`, `equipamiento`, `asignaciones_equipamiento`).
- [x] **Catálogo de Rangos SEDENA:** Insertar los 15 rangos oficiales del Ejército y Fuerza Aérea Mexicanos con `orden_jerarquico` numérico para evaluación ABAC.
- [x] **Catálogo de Secciones de Servicio:** 15 secciones orgánicas con flag `es_servicio_belico` para la sección de Materiales de Guerra.
- [x] **Inventario Representativo:** 22 ítems de equipamiento en 7 categorías (armamento nacional FX-05 Xiuhcóatl, municiones, equipo táctico, comunicaciones, vehículos, médico y supervivencia).
- [x] **Refactorización de `soldiers.rs`:** Queries con JOINs a tablas de lookup, formulario con `<select>` dinámico desde BD, campos normalizados (matrícula, apellidos separados, `rango_id`, `seccion_servicio_id`).
- [x] **Actualización de Documentación:** [docs/tursodb.md](docs/tursodb.md) actualizado con las 6 tablas del nuevo esquema.
- [x] **Pruebas Unitarias de Migraciones:** 5 tests verificando integridad estructural del SQL y presencia de datos semilla críticos.

### Iteración 2: Refactorización del Dominio en Rust y Unificación (Fase A) ✅

- [x] **Expansión del Enum `Rango`:** Actualizar de 5 a 15 variantes correspondientes a la escala oficial SEDENA, con métodos `orden_jerarquico()`, `categoria()`, `from_id()`, `to_id()` y `supera_a()`.
- [x] **Enum `CategoriaRango`:** Crear tipo que clasifica rangos en `Tropa`, `Clases`, `Oficiales`, `Jefes` y `Generales`, derivable desde `Rango::categoria()`.
- [x] **Enums de Inventario:** Crear `CategoriaEquipamiento` (7 variantes) con método `es_material_de_guerra()` y `EstadoConservacion` (`Operativo`, `EnMantenimiento`, `DeBaja`).
- [x] **Módulo ABAC (`acceso.rs`):** Implementar `ContextoAcceso` con política `puede_administrar_inventario_belico()` que evalúa rango + sección de servicio.
- [x] **Reorganización de `src/domain/`:** Migrar `types.rs` → `tipos_militares.rs`, crear `acceso.rs` e `inventario.rs`, y crear `mod.rs` con re-exportaciones aplanadas.
- [x] **Suite de Pruebas Expandida:** 14 nuevas pruebas unitarias cubriendo las 15 variantes del enum, mapeo bidireccional Enum↔BD, lógica ABAC para los escenarios de acceso.
- [x] **Unificación Idiomática (Fase A) en Controladores:** Renombrar `soldiers.rs` a `soldados.rs`, mover endpoint `/soldiers` a `/soldados` y actualizar los enlaces de navegación.

### Iteración 3: Endpoints e Integración ABAC ✅

- [x] **Endpoint de Inventario (`/inventario`):** Handler GET con listado de equipamiento en español, filtrable por categoría, con indicadores visuales de stock y selector de operador simulado.
- [x] **CRUD Protegido por ABAC:** Middleware y validación en servidor de `ContextoAcceso` antes de permitir registrar equipamiento bélico (retorna `403 Forbidden`).
- [x] **Endpoint de Asignaciones (`/asignaciones`):** Handler GET y POST para registrar la cadena de custodia de armamento y equipamiento.
- [x] **Transacciones ACID SQLite:** Descuento de stock y registro de asignación empaquetados en transacciones de base de datos atómicas.
- [x] **Pruebas de Integración:** Suite de pruebas de integración HTTP (oneshot) simulando CSRF y cookies de operador para validar las reglas ABAC de acceso y denegación.

### Iteración 4: Interfaz Maud + HTMX para Inventario ✅

- [x] **Componentización DRY con Maud:** Vistas modulares y reutilizables en `src/components/` (`alerta.rs` y `operador.rs`) desacopladas del negocio.
- [x] **Interactividad Asíncrona (HTMX Swaps):** Formularios de inventario y asignaciones 100% asíncronos y libres de recargas del navegador.
- [x] **Renderizado Condicional por ABAC:** Selectores que deshabilitan dinámicamente (`disabled`) categorías o ítems bélicos si el operador simulado no cuenta con los permisos ABAC.
- [x] **Flujo Completo de Devoluciones:** Botón "Devolver" en la bitácora que reintegra el equipo al stock en una transacción atómica protegida por ABAC.
- [x] **Fidelidad Estética (BeerCSS):** Diseño consistente con Material Design 3 e idéntico a las páginas principales.

---

## Fase 7: Hardening, Criptografía y Seguridad Operacional (Hacia la V1)

_Objetivo: Mitigar las brechas de suplantación, no-repudio y fuga de datos personales, elevando el sistema a nivel de producción seguro._

### 🚨 Auditoría de Vulnerabilidades y Brechas de Seguridad Identificadas (v0.1.x)

- [x] **VULN-01: Escalada de Privilegios y Suplantación vía Manipulación de Cookie de Operador (Crítica)**
  - **Ubicación en Código:** [soldados.rs](src/pages/soldados.rs#L422-L423) en la función `resolver_operador_activo`.
  - **Detalle Técnico:** La identidad y el contexto del operador activo se determinan leyendo la cookie en texto plano y no firmada `operador_soldado_id`.
  - **Vector de Ataque:** Cualquier usuario malintencionado con acceso al cliente puede alterar la cookie (ej. cambiar de `3` a `1`) para saltarse completamente los controles de acceso ABAC de [acceso.rs](src/domain/acceso.rs) y registrar/devolver material de guerra sin autorización real.
  - **Mitigación Requerida:** Sustituir por autenticación mediante sesión firmada y cifrada con `PrivateCookieJar` y login real.
- [x] **VULN-02: Mutabilidad de la Bitácora Histórica (Falta de No-Repudio en Cadena de Custodia)**
  - **Ubicación en Código:** [asignaciones.rs](src/pages/asignaciones.rs#L475-L483) y [asignaciones.rs](src/pages/asignaciones.rs#L670) (operaciones de inserción y actualización directa).
  - **Detalle Técnico:** La bitácora en la tabla `asignaciones_equipamiento` es mutable mediante sentencias SQL estándar y carece de verificación criptográfica de integridad.
  - **Vector de Ataque:** Un operador corrupto con acceso directo a la base de datos SQLite (`armadillos.db`) o mediante inyección SQL puede borrar o alterar registros antiguos de asignación de armas para ocultar desvíos.
  - **Mitigación Requerida:** Implementar un esquema Hash-Chain SHA-256 en la tabla donde cada registro dependa del hash del registro anterior.
- [x] **VULN-03: Exposición de Datos Personales (PII) en Reposo y Memoria**
  - **Ubicación en Código:** Tabla `soldados` (esquema DDL en [database.rs](src/db/database.rs#L68-L77)) y structs en [soldados.rs](src/pages/soldados.rs).
  - **Detalle Técnico:** Matrícula, nombres y apellidos se persisten en texto claro. Además, al procesarse en memoria, se usan strings convencionales que no se limpian de la memoria física (RAM) tras su uso.
  - **Vector de Ataque:** Un dump de memoria o un compromiso físico del archivo de la base de datos expone la estructura de personal de la unidad militar.
  - **Mitigación Requerida:** Encriptación a nivel de aplicación (ALE) con AES-256-GCM o ChaCha20-Poly1305 para columnas PII, y wrapping de variables sensibles en tipos `SecretString` (crate `secrecy`).
- [x] **VULN-04: Fuga de PII en Logs de Depuración y Observabilidad**
  - **Ubicación en Código:** [soldados.rs](src/pages/soldados.rs#L312) (`matricula = %nuevo.matricula`), [asignaciones.rs](src/pages/asignaciones.rs#L443) (`operador = %operador.nombre_completo`), [asignaciones.rs](src/pages/asignaciones.rs#L522), [asignaciones.rs](src/pages/asignaciones.rs#L628), [asignaciones.rs](src/pages/asignaciones.rs#L703) e [inventario.rs](src/pages/inventario.rs#L455).
  - **Detalle Técnico:** Las macros de eventos `tracing::info!`, `tracing::warn!` y `tracing::error!` registran directamente nombres completos de operadores y matrículas en formato JSON de texto plano.
  - **Vector de Ataque:** En entornos centralizados de recolección de logs, la PII militar viaja y se almacena en texto claro en servidores externos sin protección de identidad.
  - **Mitigación Requerida:** Crear un formateador personalizado en `tracing-subscriber` que filtre o hashee campos con PII.
- [x] **VULN-05: Exposición de Red por Ausencia de Cabeceras HTTP de Seguridad (Hardening)**
  - **Ubicación en Código:** [main.rs](src/main.rs#L130-L190) (inicialización del router Axum).
  - **Detalle Técnico:** No se configuran políticas de Content Security Policy (CSP), HTTP Strict Transport Security (HSTS) ni protección contra Clickjacking en las respuestas HTTP de Axum.
  - **Vector de Ataque:** Vulnerabilidad a ataques XSS mediante inserción de scripts JavaScript no autorizados y ataques man-in-the-middle por falta de SSL forzado.
  - **Mitigación Requerida:** Configuración de middleware global para CSP, HSTS y X-Frame-Options en el router de Axum.

### 🛠️ Tareas de Mitigación y Desarrollo V1

- [x] **Autenticación con Argon2id (Login Real):**
  - Reemplazar el simulador de operadores por un portal de autenticación real.
  - Implementar hashing de contraseñas robusto con el algoritmo **Argon2id** (usando parámetros de memoria estrictos: `m_cost = 65536` o 64MB, `t_cost = 3`, `p_cost = 4`) para mitigar ataques de diccionario y fuerza bruta a nivel local o de red.
- [x] **Sesiones Cifradas en el Servidor (`PrivateCookieJar`):**
  - Configurar **`axum-extra::extract::PrivateCookieJar`** para cifrar y firmar criptográficamente la sesión en tránsito usando una clave maestra simétrica.
  - Evitar que el cliente pueda alterar o inyectar localmente el ID del operador en la cookie, impidiendo la evasión de políticas ABAC.
- [x] **Application-Level Encryption (ALE) para PII Militar:**
  - Cifrar en origen las columnas confidenciales (`nombre`, `apellido_paterno`, `apellido_materno`, `matricula` en la tabla `soldados`, y números de serie de armamento sensible) antes de persistirse en SQLite/Turso.
  - Utilizar algoritmos criptográficos autenticados estándar como **ChaCha20-Poly1305** o **AES-256-GCM**.
  - Evitar fugas en memoria de datos sin cifrar usando el wrapper **`SecretString`** (del crate `secrecy`) para forzar la zeroización en la pila del sistema tras su uso.
- [x] **Cadena de Custodia Inmutable (Hash-Chain BLAKE3):**
  - Rediseñar la bitácora de asignaciones y devoluciones como un libro mayor append-only.
  - Cada inserción debe calcular un hash criptográfico **`SHA-256`** que concatene los datos del registro actual con el hash del registro anterior (`hash_actual = SHA-256(registro_actual || hash_anterior)`), almacenándolo en la columna `hash_verificacion`.
  - Proveer un validador de integridad asíncrono para alertar ante alteraciones maliciosas directas a nivel de base de datos.
- [x] **Políticas de Hardening de Red en Axum (CSP/HSTS):**
  - Implementar middlewares de Axum para forzar cabeceras de seguridad estrictas:
    - **Content Security Policy (CSP):** `default-src 'self'` para anular inyecciones XSS externas y prohibir JS de terceros.
    - **HTTP Strict Transport Security (HSTS):** `max-age=63072000` con `includeSubDomains` para forzar conexiones HTTPS.
    - Cabeceras secundarias: `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, y `Permissions-Policy`.
- [x] **Sanitización de Observabilidad (Logs Libres de PII):**
  - Configurar filtros personalizados en `tracing-subscriber` para ofuscar o hashear irreversiblemente matrículas y apellidos en la salida estándar de logs JSON, evitando su persistencia en recolectores de logs externos.
