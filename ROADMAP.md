# ROADMAP.md

## Fase 1: Fundamentos y Arquitectura

- [x] Estructura básica de directorios y módulos (`assets/views`, `assets/layouts`).
- [x] Configuración de `Cargo.toml` con dependencias necesarias.
- [x] Implementación de sistema de módulos (`mod.rs` en cascada).
- [ ] Auditoría de seguridad inicial (revisión de dependencias).

## Fase 2: Integración Web (Axum + Maud)

- [x] Implementación del patrón `IntoResponse` (Newtype).
- [x] Conexión entre Router, Handlers y Layouts.
- [ ] Optimización de assets estáticos (CSS/JS) mediante `tower-http`.

## Fase 3: Componentización y Modelado de Datos

- [ ] Creación de `src/assets/components/` (Componentes reutilizables).
- [ ] Definición de modelos de dominio (Structs para Soldados, Rangos, Inventario).
- [ ] Implementación de `serde` para serialización de datos.

## Fase 4: Persistencia y CRUD Seguro

- [ ] Integración de `SQLx` con `SQLite` (fase de desarrollo).
- [ ] Definición de políticas de acceso (RBAC) a nivel de modelo.
- [ ] Implementación de validaciones en los formularios de entrada.

## Fase 5: Refuerzo y Escalabilidad

- [ ] Migración de `SQLite` a `PostgreSQL`.
- [ ] Auditoría de seguridad profunda (OWASP para APIs).
- [ ] Implementación de sistema de logs auditables (quién ve qué).
