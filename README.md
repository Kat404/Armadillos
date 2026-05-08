# Armadillos - Software privado y seguro para administración de Milicia

## Propósito

Generar un software web app potente, rápido, privado y eficiente para manejar la logística, personal e inventario militar.

## Principios Fundamentales

1. **Seguridad Tipada (Type-Safety):** El sistema de tipos de Rust debe prohibir estados inválidos en la lógica militar (ej. jerarquías de mando y permisos de armas).
2. **Privacidad Total:** Arquitectura enfocada en el control absoluto de los datos, diseñando para almacenamiento local y riguroso control de acceso (RBAC).
3. **Integridad de Datos:** Uso de transacciones ACID para evitar corrupción en inventarios críticos y registros de personal.
4. **Auditabilidad:** Cada operación sobre datos sensibles debe ser trazable mediante logs estructurados.

## Stack Tecnológico

- **Lenguaje:** Rust (foco en seguridad de memoria).
- **Backend/Routing:** Axum + Tokio.
- **HTML (Plantillas):** Maud (Type-safe HTML generation).
- **Frontend Interactivo:** HTMX (Arquitectura centrada en el servidor).
- **Persistencia:** SQLite (dev) / PostgreSQL (prod) vía SQLx.
- **Serialización:** Serde.
- **Observabilidad:** Tracing (logs auditables).
- **Middleware:** Tower.
