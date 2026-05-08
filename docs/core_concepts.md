# 🧠 Conceptos Core: La Santísima Trinidad (Tokio, Axum, Tower)

Es común sentirse perdido entre estas tres bibliotecas, ya que trabajan tan juntas que sus límites parecen borrosos. Aquí tienes la explicación definitiva.

## 1. ⚡ Tokio: El Corazón Asíncrono

**¿Qué es?** Es un _runtime_ (entorno de ejecución). Rust por sí solo no sabe cómo manejar tareas asíncronas (como esperar a que llegue un paquete de red sin bloquear todo el programa).

- **Su trabajo:** Gestionar hilos de ejecución, temporizadores y entrada/salida (I/O) no bloqueante.
- **En tu código:** Lo ves en el `#[tokio::main]` y cuando usas `.await`. Sin Tokio, tu servidor no podría atender a más de una persona a la vez eficientemente.

## 2. 🧱 Tower: El Estándar de Servicios

**¿Qué es?** Es una biblioteca de componentes modulares y abstractos. Define qué es un "Servicio" en Rust: algo que recibe una petición y devuelve una respuesta de forma asíncrona.

- **Su trabajo:** Proporcionar "piezas de Lego" llamadas _Middleware_. ¿Quieres limitar el número de peticiones por segundo? ¿Quieres poner un timeout? ¿Quieres registrar (logs) cada petición? Eso es Tower.
- **En tu código:** Lo usas cuando añades `.layer(TraceLayer::new_for_http())`. Tower es la infraestructura sobre la cual se construye Axum.

## 3. 🕸️ Axum: El Framework Web

**¿Qué es?** Es una capa construida **sobre** Tokio y Tower específicamente para manejar el protocolo HTTP de forma ergonómica.

- **Su trabajo:** Enrutamiento (decidir qué función ejecuta qué URL), extracción de datos (leer JSON, formularios o parámetros de la URL) y envío de respuestas.
- **En tu código:** Es el `Router::new()` y las funciones `get()`, `post()`, etc.

---

## 💡 ¿Cómo encajan?

Imagina un restaurante:

- **Tokio** es la electricidad y el gas: sin ellos, nada funciona en segundo plano.
- **Tower** es el manual de procedimientos de los empleados: cómo deben saludar, cómo deben anotar el pedido y qué hacer si hay mucha gente.
- **Axum** es el camarero y el menú: recibe tu pedido específico ("Quiero el `/index.html`") y sabe exactamente qué cocina llamar.

---

## 🎯 ¿Por qué es "Awesome"?

Esta separación permite que si alguien inventa un nuevo sistema de logs para Tower, automáticamente funcione en Axum sin que los desarrolladores de Axum tengan que hacer nada. ¡Es la magia del código modular!
