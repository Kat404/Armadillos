# ROADMAP.md (V2: Hardened Edition)

## Fase 1: Cimientos y Seguridad Estática

_Objetivo: Establecer un entorno de desarrollo donde sea "difícil" cometer errores._

- [x] **Arquitectura Base:** Estructura de módulos y configuración de `Cargo.toml`.
- [ ] **Enforcement de Estilo y Seguridad:** Configurar `clippy` con niveles estrictos (`deny(warnings)`) y `cargo-audit` en el flujo de trabajo para detectar vulnerabilidades en dependencias.
- [ ] **Gestión de Secretos:** Implementar un sistema para que las llaves de la DB y sales de encriptación nunca toquen el código (uso de variables de entorno o `Secrecy` crate para evitar fugas en logs).
- [x] **Licenciamiento:** Definir formalmente la licencia **AGPLv3** en el repositorio para proteger el código en entornos SaaS.

## Fase 2: El Corazón del Dominio (Type-Safety Militar)

_Objetivo: Usar el sistema de tipos de Rust para que la lógica de negocio sea "imposible" de romper._

- [ ] **Modelado de Jerarquías:** Implementar la jerarquía de mando usando _State Machines_ con Enums. (Ej: Un `Soldado` solo puede transicionar a `Cabo` mediante un evento firmado).
- [ ] **Sistema de Permisos (ABAC):** Diseñar los Structs de `User` y `Context` para permitir Control de Acceso Basado en Atributos.
- [ ] **Validación en Compilación:** Asegurar que todos los modelos de `SQLx` tengan validación de tipos estricta para evitar datos corruptos en el inventario.

## Fase 3: Interfaz de Usuario Hypermedia (Maud + HTMX)

_Objetivo: Una UI rápida y privada sin el "overhead" de JavaScript._

- [x] **Pattern IntoResponse:** Newtype para manejo de respuestas limpias.
- [ ] **Componentización Crítica:** Crear `src/views/components/` para elementos de UI que requieran validación visual (ej. indicadores de nivel de acceso).
- [ ] **Seguridad en la Capa de Transporte:** Implementar protecciones CSRF mediante middlewares de Axum/Tower-HTTP, vital ya que HTMX usa peticiones AJAX.

## Fase 4: Persistencia y Cifrado en Reposo

_Objetivo: Blindar la base de datos contra accesos físicos no autorizados._

- [ ] **Estrategia de Base de Datos:** Migración controlada de SQLite a PostgreSQL usando contenedores para paridad de entornos.
- [ ] **Application-Level Encryption (ALE):** Implementar cifrado para campos sensibles (Nombres de oficiales, ubicaciones de armamento) antes de que lleguen a la DB.
- [ ] **Backups Cifrados:** Scripting (posiblemente en Nushell) para automatizar respaldos hacia almacenamiento local o remoto usando encriptación de llave pública.

## Fase 5: Auditabilidad y "Non-Repudiation"

_Objetivo: Que cada click sea rastreable y legalmente vinculante._

- [ ] **Logs Estructurados (Tracing):** Configurar `tracing-subscriber` para generar logs en formato JSON que incluyan ID de usuario y nivel de mando.
- [ ] **Integridad de Logs:** Investigar e implementar un sistema de "Hashing" de logs para detectar si alguien (incluso un admin de sistema) ha alterado los registros de acceso.
- [ ] **Hardening de Producción:** Configuración de cabeceras de seguridad (HSTS, CSP) y despliegue sobre una base de Arch Linux minimalista o contenedores Alpine.

---
