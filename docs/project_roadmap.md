# 🚀 Roadmap: Construyendo Armadillos paso a paso

Si te sientes abrumado, sigue este camino. Está diseñado para que vayas de menos a más, asegurando que cada pieza funcione antes de pasar a la siguiente.

## Fase 1: El Esqueleto (Base)

1.  `[x]` **Limpieza del `main.rs`**: Deja solo la configuración del servidor y una ruta de prueba.
2.  `[x]` **Configuración de Logs**: Asegúrate de que `tracing` esté funcionando para ver quién entra a tu web en la terminal.
3.  `[x]` **Archivos Estáticos**: Crea una carpeta `assets/` y configura Axum para servir tu `style.css`. ¡Pruébalo entrando a `localhost:3000/assets/style.css`!

## Fase 2: La Piel (UI con Maud)

1.  `[x]` **Crear el Layout**: Define la función `layout` en un nuevo archivo (ej: `src/views.rs`). Debe incluir el CSS y una estructura HTML básica.
2.  `[x]` **Página de Inicio**: Transforma tu `home_handler` para que devuelva un `Markup` de Maud usando el layout.
3.  `[x]` **Componentes**: Crea pequeñas funciones que devuelvan `Markup` para partes repetitivas (botones, tarjetas, menús).

## Fase 3: Los Nervios (Enrutamiento y Datos)

1.  `[x]` **Rutas Dinámicas**: Crea una página que reciba un nombre por la URL (ej: `/hola/jose`) y lo muestre usando Maud.
2.  `[x]` **Manejo de Formularios**: Crea una página con un `<form>`, recíbelo en Axum y procesa los datos.
3.  `[x]` **Estado Compartido**: Aprende a usar `State` para pasar datos entre rutas sin usar globales (ej: un contador de visitas).

## Fase 4: El Cerebro (Lógica de Negocio)

1.  `[x]` **Base de Datos (Opcional)**: Integra una base de datos sencilla (como SQLite con `sqlx`).
2.  `[ ]` **Módulos**: Organiza tu código. No dejes todo en `main.rs`. Separa en `routes.rs`, `views.rs` y `models.rs`.

---

## 🛠️ Herramientas de Diagnóstico

Como usuario de **Arch Linux** y **Nushell**, tienes herramientas poderosas:

```nu
# Monitorear cambios y recompilar automáticamente (Instala: cargo install cargo-watch)
cargo watch -x run

# Ver peticiones HTTP en tiempo real con curl
curl -v localhost:3000

# Analizar dependencias y su tamaño
cargo tree
```

---

## 🏁 Meta Final

Un servidor web escrito en Rust que sea:

- **Rápido:** Gracias a Tokio y Maud.
- **Seguro:** Gracias al sistema de tipos de Rust.
- **Mantenible:** Gracias a la arquitectura modular de Axum y Tower.

¡Tú puedes, futuro ingeniero! 🐾🦀
