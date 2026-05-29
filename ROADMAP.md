# ROADMAP.md (V2: Hardened Edition)

## Fase 1: Cimientos y Seguridad Estática

_Objetivo: Establecer un entorno de desarrollo donde sea "difícil" cometer errores._

- [x] **Arquitectura Base:** Estructura de módulos y configuración de `Cargo.toml`.
- [x] **Enforcement de Estilo y Seguridad:** Configurar `clippy` con niveles estrictos (`deny(warnings)`), formateo automático e integración de pruebas unitarias obligatorias en [GEMINI.md](file:///home/josel/Proyectos/Armadillos/GEMINI.md).
- [x] **Gestión de Secretos:** Implementar un sistema para que las llaves de la DB y sales de encriptación nunca toquen el código (uso de variables de entorno con fallbacks controlados en desarrollo).
- [x] **Licenciamiento:** Definir formalmente la licencia **AGPLv3** en el repositorio para proteger el código en entornos SaaS.

## Fase 2: El Corazón del Dominio (Type-Safety Militar)

_Objetivo: Usar el sistema de tipos de Rust para que la lógica de negocio sea "imposible" de romper._

- [x] **Modelado de Jerarquías:** Implementar la jerarquía de mando usando _State Machines_ con Enums (`Rango`, `EstadoServicio`).
- [ ] **Sistema de Permisos (ABAC):** Diseñar los Structs de `User` y `Context` para permitir Control de Acceso Basado en Atributos.
- [x] **Validación de Tipos (Dominio):** Eliminar la persistencia de strings crudos ("Stringly-Typed") mapeando e insertando tipos seguros de datos en los endpoints.
- [x] **Pruebas de Lógica de Dominio:** Implementar suite de pruebas unitarias (`cargo test`) para validar las reglas de la máquina de estados militar y de seguridad.

## Fase 3: Interfaz de Usuario Hypermedia (Maud + HTMX)

_Objetivo: Una UI rápida y privada sin el "overhead" de JavaScript._

- [x] **Pattern IntoResponse:** Newtype para manejo de respuestas limpias.
- [x] **Interactividad Hypermedia con HTMX y Maud:** Implementación inicial de flujo CRUD asíncrono para gestión de personal militar (`/soldiers`).
- [x] **Autocontención de Assets:** Descargar y servir localmente `htmx.min.js` y `material-dynamic-colors.min.js` en [main_layout.rs](file:///home/josel/Proyectos/Armadillos/src/layouts/main_layout.rs), removiendo la dependencia de CDNs.
- [ ] **Componentización Crítica:** Crear `src/views/components/` para elementos de UI que requieran validación visual (ej. indicadores de nivel de acceso).
- [x] **Seguridad en la Capa de Transporte:** Implementar protecciones CSRF mediante middlewares de Axum/Tower-HTTP, vital ya que HTMX usa peticiones AJAX.

## Fase 4: Persistencia y Cifrado en Reposo

_Objetivo: Blindar la base de datos contra accesos físicos no autorizados._

- [x] **Estrategia de Base de Datos (Segura y Local-First):** Integración de Turso DB (reescrita en Rust) en modo local-first en `armadillos.db` para evitar FFI unsafe de C, y documentación del flujo en [docs/tursodb.md](docs/tursodb.md).
- [x] **Cifrado de Base de Datos TursoDB:** Configurar el cifrado de página nativo en reposo de Turso (`Aegis256`) utilizando la clave del sistema.
- [ ] **Application-Level Encryption (ALE):** Implementar cifrado para campos sensibles (Nombres de oficiales, ubicaciones de armamento) antes de que lleguen a la DB.
- [ ] **Backups Cifrados:** Scripting (posiblemente en Nushell) para automatizar respaldos hacia almacenamiento local o remoto usando encriptación de llave pública.

## Fase 5: Auditabilidad y "Non-Repudiation"

_Objetivo: Que cada click sea rastreable y legalmente vinculante._

- [x] **Logs Estructurados (Tracing):** Configurar `tracing-subscriber` para generar logs en formato JSON estructurado, instrumentando handlers críticos con spans asíncronos.
- [ ] **Integridad de Logs:** Investigar e implementar un sistema de "Hashing" de logs para detectar si alguien (incluso un admin de sistema) ha alterado los registros de acceso.
- [ ] **Hardening de Producción:** Configuración de cabeceras de seguridad (HSTS, CSP) y despliegue sobre una base de Arch Linux minimalista o contenedores Alpine.

---
