# 🐾 Proyecto Armadillos: Documentación "Awesome"

¡Bienvenido a la documentación oficial de **Armadillos**! Este recurso está diseñado para guiarte desde los conceptos más básicos hasta el despliegue de una aplicación web robusta y tipada en Rust.

---

## 🏗️ El Stack de Armadillos

Nuestra aplicación se basa en cuatro pilares fundamentales del ecosistema de Rust:

| Herramienta | Función | Analogía |
| :--- | :--- | :--- |
| **Tokio** | Runtime Asíncrono | El motor de la fábrica (maneja hilos y tareas). |
| **Axum** | Framework Web | El recepcionista (recibe peticiones y las envía al lugar correcto). |
| **Tower** | Capa de Servicios | Los protocolos de seguridad y logística (middleware). |
| **Maud** | Motor de Plantillas | El diseñador de interiores (genera HTML ultra rápido y tipado). |

---

## 📚 Guías de Aprendizaje

Para empezar, sigue este orden recomendado:

1.  [**Conceptos Core (Tokio vs Axum vs Tower)**](core_concepts.md): Entiende la jerarquía de tu servidor.
2.  [**Rutas y Handlers**](routing_and_handlers.md): Cómo definir URLs y qué datos recibir de ellas.
3.  [**Templating con Maud**](templating_with_maud.md): Cómo integrar HTML en Rust sin perder el tipado.
4.  [**Roadmap del Proyecto**](project_roadmap.md): Un paso a paso sugerido para construir tu aplicación.

---

## 🛠️ Comandos Útiles (Nushell/Rust)

Para moverte por este proyecto como un profesional:

```nu
# Correr el servidor en modo desarrollo
cargo run

# Checar errores sin compilar (muy rápido)
cargo check

# Formatear el código según los estándares
cargo fmt
```

---

> [!TIP]
> Recuerda que en Rust, si compila, probablemente funciona. No tengas miedo a los errores del compilador; son tus mejores amigos para evitar bugs en producción.
