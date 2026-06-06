# 🚀 Roadmap del Proyecto: Armadillos

El desarrollo del proyecto **Armadillos** se ha planeado en 7 fases estratégicas con el fin de construir una aplicación militar escalable, privada y robusta basada en Rust, TursoDB y Maud.

---

## Fases de Desarrollo

### 🏗️ Fase 1: Cimientos y Seguridad Estática (100% Completada)

- Configuración de arquitectura limpia, lints estrictos en `clippy` e integración de licencias **AGPLv3**.
- Definición y gestión de secretos y llaves simétricas a través de variables de entorno seguras.

### 🧠 Fase 2: Corazón del Dominio (100% Completada)

- Modelado de tipos y jerarquías militares seguras mediante máquinas de estados en Rust (`domain::Rango`, `domain::EstadoServicio`).
- Diseño e implementación de la política de Control de Acceso Basado en Atributos (ABAC) para evaluar roles de personal.

### 🎨 Fase 3: Interfaz Maud + HTMX y Middleware (100% Completada)

- Renderizado HTML rápido y tipado con **Maud** y navegación asíncrona fluida con **HTMX**.
- Creación de componentes reutilizables y mitigación de XSS/Clickjacking mediante middlewares HTTP y tokens CSRF.

### 💾 Fase 4: Persistencia y Cifrado de BD (100% Completada)

- Integración de Turso DB en modalidad local-first (`armadillos.db`).
- Configuración de cifrado de página simétrico en reposo (`Aegis256`) y backups cifrados asimétricos usando **`age`** y **Nushell**.

### 📊 Fase 5: Observabilidad y Auditoría (80% Completada)

- Registro estructurado asíncrono con `tracing-subscriber` en formato JSON.
- Hardening de red en producción e instrumentación de trazabilidad de flujo de solicitudes.

### 📦 Fase 6: Inventario Militar y Normalización (100% Completada)

- Normalización del esquema de base de datos a la **Tercera Forma Normal (3FN)** con 6 tablas.
- Integración del catálogo de rangos SEDENA oficiales e inventario de material bélico.
- Implementación de flujos de asignación y devoluciones en caliente de armería.

### 🛡️ Fase 7: Mitigaciones de Seguridad y V1 (100% Completada)

- Portal de autenticación real con cifrado **Argon2id**.
- Manejo de sesiones en el servidor mediante cookies firmadas y cifradas (`PrivateCookieJar`).
- Encriptación a nivel de aplicación (ALE) para PII con **ChaCha20-Poly1305** y **Búsquedas Ciegas (Blind Index)** con BLAKE3.
- Bitácora inmutable con encadenamiento criptográfico (**Hash-Chain**) para no-repudio.
- Sanitización de eventos de observabilidad (logs libres de PII).
