# 💾 Guía de Respaldos Cifrados: Armadillos

Este documento describe la estrategia y los procedimientos para realizar copias de seguridad consistentes en caliente y cifradas de la base de datos **Armadillos** (`armadillos.db`).

---

## 1. Copias Consistentes (Evitando Inconsistencias en WAL)

Dado que la base de datos SQLite opera en modo **WAL** (Write-Ahead Logging) para permitir concurrencia de lecturas y escrituras sin bloqueos:

- **Riesgo:** Copiar directamente el archivo `armadillos.db` mediante comandos como `cp` puede dar como resultado un archivo corrupto o incompleto si hay transacciones activas o cambios no sincronizados en el diario WAL (`armadillos.db-wal`).
- **Solución:** Utilizamos la instrucción nativa `VACUUM INTO` a través del cliente oficial de `sqlite3`:

  ```bash
  sqlite3 armadillos.db "VACUUM INTO 'copia_temporal.db';"
  ```

  Esto crea una copia exacta, consistente, desfragmentada y completa de la base de datos en caliente, sin bloquear el servidor web activo.

---

## 2. Cifrado Asimétrico con `age`

Los respaldos de la base de datos contienen información militar y de personal sensible. Su almacenamiento en frío requiere encriptación asimétrica obligatoria.

- **Herramienta:** Utilizamos **`age`** (Actually Good Encryption), un reemplazo de GPG moderno, seguro y KISS.
- **Flujo:**
  1. Se cifra el archivo temporal consistente usando la clave pública de los administradores autorizados.
  2. La clave privada se mantiene en un almacenamiento físico seguro fuera del servidor de base de datos.
  3. El archivo original temporal no cifrado se elimina inmediatamente del disco de forma segura (zeroización local).

---

## 3. Automatización con Nushell (`backup.nu`)

Para facilitar y estructurar el proceso, desarrollamos el script de automatización [backup.nu](../../scripts/backup.nu) escrito en Nushell.

### Parámetros del Script

El script acepta tres parámetros configurables:

- `--db-path`: Ruta a la base de datos (por defecto: `armadillos.db`).
- `--backup-dir`: Directorio destino de respaldos (por defecto: `backups`).
- `--key-file`: Ruta al archivo de clave de `age`. Si no se provee, el script buscará o generará automáticamente un par de claves local usando `age-keygen` en el directorio de backups.

### Ejemplo de Ejecución en Nushell

Para correr el script en el entorno de Nushell de desarrollo:

```nu
# Ejecutar respaldo de la base de datos por defecto
nu scripts/backup.nu

# Ejecutar respaldo especificando un directorio y clave personalizada
nu scripts/backup.nu --backup-dir "/var/backups/armadillos" --key-file "~/.ssh/id_ed25519.pub"
```

---

## 4. Registro Estructurado (Auditoría de Respaldos)

El script registra de manera estructurada cada intento de respaldo en el archivo JSON `backups_log.json` dentro de la carpeta de respaldos.

Ejemplo de estructura del log JSON:

```json
[
  {
    "timestamp": "2026-06-06 17:42:00",
    "database": "armadillos.db",
    "backup_file": "backups/backup_20260606_174200.db.age",
    "original_size_bytes": 147456,
    "encrypted_size_bytes": 147524,
    "public_key_used": "age1q42...",
    "success": true
  }
]
```

Este archivo permite monitorear de forma automatizada mediante scripts si los backups se están realizando con la periodicidad requerida.
