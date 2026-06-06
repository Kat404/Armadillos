# 🛡️ Guía de Seguridad y Criptografía: Armadillos

Este documento detalla la arquitectura de seguridad, los controles de acceso y los esquemas criptográficos implementados en **Armadillos** para proteger la cadena de custodia de armamento y la confidencialidad de la información militar.

---

## 1. Autenticación Militar (Argon2id)

Para mitigar ataques de diccionario y fuerza bruta local o de red, implementamos el algoritmo de derivación de claves estándar **Argon2id** (perfil SEDENA/FOSS) mediante la biblioteca `argon2`.

- **Parámetros Estrictos de Memoria:**
  - `m_cost` = 65,536 (64 MB de memoria física requerida para cómputo del hash).
  - `t_cost` = 3 iteraciones.
  - `p_cost` = 4 hilos en paralelo.
- **Salt Aleatorio:** Cada contraseña utiliza un salt único criptográficamente seguro de 128 bits generado mediante `rand::rngs::OsRng`.

---

## 2. Gestión de Sesiones Seguras (Private Cookies)

La identidad del operador activo en sesión y el contexto ABAC se transmiten cifrados y firmados simétricamente.

- **Extractor `PrivateCookieJar`:**
  - La cookie `operador_soldado_id` se encripta y se firma utilizando la clave maestra simétrica configurada en la variable de entorno `ARMADILLOS_ALE_KEY`.
  - Impide la manipulación y la inyección local del identificador de operador desde el navegador, anulando vectores de ataque de escalada de privilegios (VULN-01).

---

## 3. Cifrado a Nivel de Aplicación (ALE) y Protección en Memoria

Para mitigar **VULN-03** (Exposición de Datos Personales o PII en reposo), ciframos los datos sensibles de los soldados (`matricula`, `nombre`, `apellido_paterno`, `apellido_materno`) antes de que toquen la base de datos física.

- **Algoritmo:** **ChaCha20-Poly1305 AEAD** (Authenticated Encryption with Associated Data).
- **Nonce Único:** Cada cifrado genera un nonce aleatorio de 96 bits (`OsRng`).
- **Formato en Base de Datos:** `Nonce (12 bytes) + Cifrado + Tag de Autenticación (16 bytes)` concatenados y codificados en formato hexadecimal.
- **Protección RAM (`secrecy`):**
  - Los datos personales en texto claro extraídos se encapsulan en tipos `SecretString`.
  - Este tipo zeroiza (sobrescribe con ceros) el búfer de memoria inmediatamente después de salir de ámbito y enmascara el renderizado en consola/debug con `[REDACTED]`.

---

## 4. Índice Ciego (Blind Index)

Para posibilitar consultas directas (`SELECT`) y restricciones de clave única (`UNIQUE`) sobre campos cifrados sin realizar descifrado en memoria de todas las filas:

- **Fórmula:**
  $$\text{blind\_index} = \text{BLAKE3\_Keyed}(\text{matricula}, \text{salt\_secreto})$$
- **Implementación:**
  - Usamos `blake3::keyed_hash` con la clave simétrica maestra de 256 bits (`ARMADILLOS_ALE_KEY`).
  - El resultado hexadecimal se guarda en `matricula_blind_index`. Las búsquedas hashean la entrada del usuario y la comparan directamente en SQLite mediante un índice B-Tree indexado y rápido.

---

## 5. Bitácora Inmutable (Hash-Chain)

Para prevenir la alteración de registros antiguos de la cadena de custodia (VULN-02):

- **Cadena de Bloques Criptográfica:**
  - Cada registro de la tabla `asignaciones_equipamiento` incluye la columna `hash_verificacion`.
  - El hash del evento actual se calcula concatenando criptográficamente los campos del evento con el hash del registro anterior (`hash_anterior`):
    $$\text{hash\_actual} = \text{BLAKE3}(\text{tipo\_evento} \mathbin{\Vert} \text{soldado\_id} \mathbin{\Vert} \text{equipamiento\_id} \mathbin{\Vert} \text{cantidad} \mathbin{\Vert} \text{fecha} \mathbin{\Vert} \text{operador\_id} \mathbin{\Vert} \text{origen\_id} \mathbin{\Vert} \text{hash\_anterior})$$
- **Resiliencia de Historial:**
  - Si un administrador del sistema o un atacante altera directamente una asignación o devolución en `armadillos.db`, la cadena se rompe y los algoritmos de verificación detectan la incongruencia de inmediato.

---

## 6. Hardening del Servidor (Cabeceras HTTP)

Configuración global en el Router de Axum para endurecer el transporte HTTP:

- **Content Security Policy (CSP):** `default-src 'self'`. Desactiva la ejecución de scripts no autorizados o CDNs externas.
- **HTTP Strict Transport Security (HSTS):** `max-age=63072000; includeSubDomains`. Fuerza el uso de HTTPS en los navegadores.
- **X-Frame-Options:** `DENY`. Evita ataques de Clickjacking.
- **X-Content-Type-Options:** `nosniff`. Evita ataques de suplantación MIME.

---

## 7. Sanitización en Logs de Observabilidad (PII Redacted)

Para mitigar **VULN-04** (fuga de PII en recolectores de logs externos):

- Se prohíbe pasar cadenas en texto claro (como matrícula o nombres) en los spans de `tracing`.
- Las macros de log registran únicamente el `operador_id` numérico, el rango general y el `matricula_blind_index`.
- Las variables sensibles se imprimen usando el wrapping `SecretString`, garantizando que aparezcan como `[REDACTED]` ante cualquier vaciado de depuración accidental.
