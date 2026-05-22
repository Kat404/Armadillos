# 🛡️ GEMINI.md - Directrices de Desarrollo y Operación de Armadillos

Este documento define el perfil de identidad, las directrices de seguridad de la información y el flujo de trabajo crítico para cualquier modelo de Inteligencia Artificial u operador que modifique o analice el codebase de **Armadillos**.

---

## 👤 Perfil del Proyecto & Filosofía

- **Propósito:** Software All-in-One (FOSS) privado y seguro para la administración militar (inventarios, jerarquía y personal).
- **Licenciamiento:** Licencia **AGPLv3** obligatoria. Toda contribución debe respetar esta naturaleza libre y auditable.
- **Enfoque de Ingeniería:**
  - **KISS (Keep It Simple, Stupid):** Evitar monolitos hiper-complejos o sobre-ingeniería de patrones.
  - **DIY (Do It Yourself):** Preferir soluciones autocontenidas y entender el flujo completo del stack en lugar de añadir crates de terceros sin auditar.
  - **Type-Safety Militar:** Usar el compilador de Rust para validar jerarquías de mando y permisos de armas en tiempo de compilación.

---

## ⚙️ Reglas Críticas de Interacción y Código

### 1. Seguridad de Memoria y Tipado

- **Cero Código Inseguro (`unsafe`):** Queda estrictamente prohibido el uso de bloques `unsafe` a menos que exista una justificación de rendimiento crítico validada rigurosamente.
- **Jerarquías Basadas en Tipos:** Utilizar _State Machines_ y Enums en Rust para transiciones de roles (ej. transicionar de `Soldado` a `Cabo` requiere validaciones explícitas imposibles de omitir en código).

### 2. Privacidad y Seguridad en Datos

- **Gestión de Secretos:** Las credenciales de bases de datos y sales de encriptación nunca deben persistir en el código fuente. Utilizar variables de entorno y el crate `secrecy` para prevenir fugas accidentales en buffers o logs de depuración.
- **Application-Level Encryption (ALE):** Datos altamente confidenciales (ej. nombres de oficiales, ubicaciones de armamento crítico) deben ser encriptados antes de persistirse en la base de datos (PostgreSQL/SQLite).
- **Cabeceras de Seguridad y Red:**
  - Configurar políticas estrictas de CSP (Content Security Policy) para evitar inyecciones.
  - Asegurar HSTS (HTTP Strict Transport Security).

### 3. Observabilidad sin Fugas

- **Logs Estructurados:** Utilizar `tracing` para emitir logs en formato JSON estructurado.
- **Filtrado de PII:** Asegurarse de que ninguna información personalmente identificable (PII) o secreto de seguridad sea capturado en los logs.

### 4. Stack de Interfaz Hypermedia (Maud + HTMX)

- **Cero Bloatware de JS:** Evitar frameworks pesados de frontend (React, Vue). La interactividad se gestiona en el servidor con **HTMX** y el renderizado tipado se realiza mediante **Maud**.
- **Componentización DRY:** Crear componentes reutilizables en Rust usando funciones y macros de Maud en `src/components`.

### 5. Convenciones de Rust

- Enforzar las directrices idiomáticas de Rust. Denunciar módulos o variables que no sigan las convenciones (ej. nombres de módulos en PascalCase en lugar de `snake_case`).
- Todo código antes de ser entregado debe pasar:

  ```bash
  cargo clippy --all-targets -- -D warnings
  cargo fmt --check
  ```

---

## 🛠️ Flujo de Trabajo y Entorno (Nushell Target)

- **Target Shell del Usuario:** Nushell (`nu`). Toda herramienta de automatización, scripts de backups cifrados o comandos de infraestructura locales deben ser escritos y estructurados en formato compatible con Nushell.
- **Core Tooling del Sistema (Arch Linux):**
  - Redes: `iproute2` e `nftables`.
  - Búsqueda: `rg` (ripgrep) y `fd`.
  - Visualización: `bat` y `eza`.
  - Interactividad: `fzf`.

---

## 📖 Mandato Ético para IA y Colaboradores

1. **Explicar el "Por qué" antes del "Cómo":** No se aceptarán parches de código mecánicos sin una explicación analítica de su impacto en la seguridad de memoria, privacidad de datos o rendimiento.
2. **Denuncia Técnica Activa:** Como colaboradores del proyecto, se debe actuar con rigor técnico. Cualquier violación a SOLID, DRY, KISS o la introducción de potenciales vulnerabilidades debe ser denunciada con dureza técnica y acompañada de su refactorización óptima.
