//! Módulo de migraciones para inicializar el esquema de la base de datos.
//!
//! **¿Por qué un módulo separado?**
//! Centralizar las sentencias DDL y los datos semilla aquí permite:
//! 1. Evolucionar el esquema sin contaminar `main.rs`.
//! 2. Auditar cambios estructurales en un solo archivo.
//! 3. Facilitar la escritura de pruebas de integración contra un esquema limpio.
//!
//! Todas las tablas están diseñadas en **Tercera Forma Normal (3FN)** para
//! eliminar redundancias y dependencias transitivas, garantizando integridad
//! referencial y eficiencia en las consultas de TursoDB (libsql/SQLite).

use turso::Connection;

/// SQL para crear todas las tablas del esquema normalizado.
///
/// **Decisiones de diseño:**
/// - `orden_jerarquico` en `rangos`: Permite evaluar permisos con una simple
///   comparación numérica (`>=`) en lugar de cadenas de `match` frágiles.
/// - `es_servicio_belico` en `secciones_servicios`: Flag booleano que, junto
///   con el rango, alimenta la lógica ABAC para acceso al inventario de guerra.
/// - `codigo_inventario` en `equipamiento`: Clave natural UNIQUE para trazabilidad
///   física del material, independiente del ID autoincremental interno.
/// - `autorizado_por_soldado_id` en `asignaciones_equipamiento`: Implementa
///   la cadena de custodia — quién autorizó la entrega de material.
const SCHEMA_SQL: &str = "
-- ============================================================
-- TABLA: rangos
-- Catálogo de rangos militares mexicanos (SEDENA y SEMAR).
-- Normalización: Tabla de lookup independiente (1FN-3FN).
-- ============================================================
CREATE TABLE IF NOT EXISTS rangos (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre              TEXT    NOT NULL UNIQUE,
    categoria           TEXT    NOT NULL,
    orden_jerarquico    INTEGER NOT NULL UNIQUE
);

-- ============================================================
-- TABLA: secciones_servicios
-- Catálogo de secciones/servicios orgánicos de un plantel.
-- El flag 'es_servicio_belico' determina si el personal de
-- clases (Sargentos) adscrito tiene acceso al inventario de
-- material de guerra (lógica ABAC).
-- ============================================================
CREATE TABLE IF NOT EXISTS secciones_servicios (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre              TEXT    NOT NULL UNIQUE,
    es_servicio_belico  INTEGER NOT NULL DEFAULT 0
);

-- ============================================================
-- TABLA: soldados
-- Registro de personal militar. Vincula al rango y sección
-- mediante claves foráneas, eliminando dependencias transitivas.
-- La matrícula original se encripta y se indexa mediante 'matricula_blind_index'.
-- ============================================================
CREATE TABLE IF NOT EXISTS soldados (
    id                          INTEGER PRIMARY KEY AUTOINCREMENT,
    matricula_blind_index       TEXT    NOT NULL UNIQUE,
    matricula_encriptada        TEXT    NOT NULL,
    nombre_encriptado           TEXT    NOT NULL,
    apellido_paterno_encriptado TEXT    NOT NULL,
    apellido_materno_encriptado TEXT,
    rango_id                    INTEGER NOT NULL,
    seccion_servicio_id         INTEGER NOT NULL,
    estado                      TEXT    NOT NULL DEFAULT 'Activo',
    FOREIGN KEY (rango_id)            REFERENCES rangos(id),
    FOREIGN KEY (seccion_servicio_id)  REFERENCES secciones_servicios(id)
);


-- ============================================================
-- TABLA: categorias_equipamiento
-- Clasifica el equipamiento en categorías. El flag
-- 'es_material_de_guerra' controla el acceso restringido.
-- ============================================================
CREATE TABLE IF NOT EXISTS categorias_equipamiento (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre                  TEXT    NOT NULL UNIQUE,
    es_material_de_guerra   INTEGER NOT NULL DEFAULT 0
);

-- ============================================================
-- TABLA: equipamiento
-- Inventario físico del plantel. Cada ítem tiene un código
-- de inventario único para trazabilidad. El stock_disponible
-- se mantiene separado del stock_total para reflejar
-- asignaciones activas sin recalcular.
-- ============================================================
CREATE TABLE IF NOT EXISTS equipamiento (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    codigo_inventario       TEXT    NOT NULL UNIQUE,
    nombre                  TEXT    NOT NULL,
    descripcion             TEXT,
    categoria_id            INTEGER NOT NULL,
    estado_conservacion     TEXT    NOT NULL DEFAULT 'Operativo',
    stock_total             INTEGER NOT NULL DEFAULT 0,
    stock_disponible        INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (categoria_id) REFERENCES categorias_equipamiento(id)
);

-- ============================================================
-- TABLA: asignaciones_equipamiento (LEDGER APPEND-ONLY)
-- Libro mayor inmutable de la cadena de custodia: cada evento
-- (ASIGNACION o DEVOLUCION) se registra como un INSERT.
-- Nunca se realizan UPDATE ni DELETE sobre esta tabla.
-- 'hash_verificacion' encadena criptográficamente cada registro
-- al anterior mediante BLAKE3 para garantizar no-repudio.
-- ============================================================
CREATE TABLE IF NOT EXISTS asignaciones_equipamiento (
    id                          INTEGER PRIMARY KEY AUTOINCREMENT,
    tipo_evento                 TEXT    NOT NULL,
    soldado_id                  INTEGER NOT NULL,
    equipamiento_id             INTEGER NOT NULL,
    cantidad                    INTEGER NOT NULL DEFAULT 1,
    fecha_evento                TEXT    NOT NULL DEFAULT (datetime('now')),
    autorizado_por_soldado_id   INTEGER NOT NULL,
    asignacion_origen_id        INTEGER,
    hash_verificacion           TEXT    NOT NULL,
    FOREIGN KEY (soldado_id)                REFERENCES soldados(id),
    FOREIGN KEY (equipamiento_id)           REFERENCES equipamiento(id),
    FOREIGN KEY (autorizado_por_soldado_id) REFERENCES soldados(id),
    FOREIGN KEY (asignacion_origen_id)      REFERENCES asignaciones_equipamiento(id)
);

-- ============================================================
-- TABLA: credenciales_soldados
-- Almacena las contraseñas hasheadas y nombres de usuario para
-- el acceso autenticado del personal.
-- ============================================================
CREATE TABLE IF NOT EXISTS credenciales_soldados (
    soldado_id      INTEGER PRIMARY KEY,
    usuario         TEXT    NOT NULL UNIQUE,
    password_hash   TEXT    NOT NULL,
    FOREIGN KEY (soldado_id) REFERENCES soldados(id) ON DELETE CASCADE
);
";

/// Datos semilla: Rangos oficiales del Ejército y Fuerza Aérea Mexicanos
/// según la Ley Orgánica del Ejército y Fuerza Aérea Mexicanos y las
/// equivalencias publicadas por la SEDENA/SEMAR.
///
/// **¿Por qué insertar rangos como datos semilla y no como un Enum puro?**
/// Porque la base de datos necesita referencias de clave foránea (`rango_id`)
/// estables. El Enum en Rust (`domain::Rango`) sigue existiendo para
/// validación en tiempo de compilación, pero su mapeo a la BD requiere filas
/// persistidas con IDs predecibles.
const SEED_RANGOS_SQL: &str = "
INSERT OR IGNORE INTO rangos (nombre, categoria, orden_jerarquico) VALUES
    ('Soldado',             'Tropa',     1),
    ('Soldado de Primera',  'Tropa',     2),
    ('Cabo',                'Clases',    3),
    ('Sargento Segundo',    'Clases',    4),
    ('Sargento Primero',    'Clases',    5),
    ('Subteniente',         'Oficiales', 6),
    ('Teniente',            'Oficiales', 7),
    ('Capitán Segundo',     'Oficiales', 8),
    ('Capitán Primero',     'Oficiales', 9),
    ('Mayor',               'Jefes',     10),
    ('Teniente Coronel',    'Jefes',     11),
    ('Coronel',             'Jefes',     12),
    ('General Brigadier',   'Generales', 13),
    ('General de Brigada',  'Generales', 14),
    ('General de División', 'Generales', 15);
";

/// Datos semilla: Secciones y servicios orgánicos de un plantel militar.
/// El flag `es_servicio_belico` marca las secciones cuyo personal de clases
/// (Sargentos) obtiene acceso CRUD al inventario de material de guerra.
const SEED_SECCIONES_SQL: &str = "
INSERT OR IGNORE INTO secciones_servicios (nombre, es_servicio_belico) VALUES
    ('Estado Mayor',                 0),
    ('Infantería',                   0),
    ('Caballería',                   0),
    ('Artillería',                   0),
    ('Ingenieros',                   0),
    ('Materiales de Guerra',         1),
    ('Transmisiones',                0),
    ('Transportes',                  0),
    ('Administración',               0),
    ('Intendencia',                  0),
    ('Sanidad',                      0),
    ('Justicia Militar',             0),
    ('Veterinaria y Remonta',        0),
    ('Meteorología Militar',         0),
    ('Informática',                  0);
";

/// Datos semilla: Categorías de equipamiento con su flag de control bélico.
const SEED_CATEGORIAS_SQL: &str = "
INSERT OR IGNORE INTO categorias_equipamiento (nombre, es_material_de_guerra) VALUES
    ('Armamento',                    1),
    ('Municiones',                   1),
    ('Equipo Táctico Individual',    0),
    ('Comunicaciones y Óptica',      0),
    ('Vehículos y Transporte',       0),
    ('Equipo Médico',                0),
    ('Supervivencia y Campaña',      0);
";

/// Datos semilla: Equipamiento representativo de un plantel militar mexicano.
/// Incluye ítems de fabricación nacional (FX-05 Xiuhcóatl, Fusil Morelos)
/// y equipo estándar de dotación.
const SEED_EQUIPAMIENTO_SQL: &str = "
INSERT OR IGNORE INTO equipamiento (codigo_inventario, nombre, descripcion, categoria_id, estado_conservacion, stock_total, stock_disponible) VALUES
    -- Armamento (categoria_id = 1)
    ('ARM-001', 'Fusil FX-05 Xiuhcóatl',       'Fusil de asalto calibre 5.56x45mm NATO. Diseño y fabricación DGIM.',                  1, 'Operativo', 150, 150),
    ('ARM-002', 'Fusil HK G3',                  'Fusil de batalla calibre 7.62x51mm NATO.',                                            1, 'Operativo', 80,  80),
    ('ARM-003', 'Pistola Beretta 92FS',          'Pistola semiautomática calibre 9x19mm Parabellum.',                                   1, 'Operativo', 100, 100),
    ('ARM-004', 'Fusil de Precisión Morelos',    'Fusil de precisión de fabricación nacional DGIM.',                                    1, 'Operativo', 20,  20),
    ('ARM-005', 'Ametralladora HK21',            'Ametralladora ligera calibre 7.62x51mm NATO.',                                       1, 'Operativo', 30,  30),

    -- Municiones (categoria_id = 2)
    ('MUN-001', 'Cartucho 5.56x45mm NATO',       'Lote de cartuchos para fusiles FX-05. Caja de 1000 unidades.',                       2, 'Operativo', 500, 500),
    ('MUN-002', 'Cartucho 9x19mm Parabellum',    'Lote de cartuchos para pistolas Beretta. Caja de 500 unidades.',                     2, 'Operativo', 300, 300),
    ('MUN-003', 'Cartucho 7.62x51mm NATO',       'Lote de cartuchos para fusiles HK G3 y HK21. Caja de 500 unidades.',                2, 'Operativo', 200, 200),

    -- Equipo Táctico Individual (categoria_id = 3)
    ('TAC-001', 'Casco Balístico Kevlar',         'Casco de protección balística nivel IIIA.',                                         3, 'Operativo', 200, 200),
    ('TAC-002', 'Chaleco Antibalas Nivel IV',     'Chaleco con placas cerámicas de protección balística nivel IV.',                     3, 'Operativo', 200, 200),
    ('TAC-003', 'Mochila Táctica de Campaña',     'Mochila de 72 horas con sistema MOLLE.',                                            3, 'Operativo', 180, 180),
    ('TAC-004', 'Uniforme Pixelado Selva',        'Uniforme de campaña patrón pixelado selva.',                                         3, 'Operativo', 300, 300),

    -- Comunicaciones y Óptica (categoria_id = 4)
    ('COM-001', 'Radio Táctico Harris',           'Radio portátil con encriptación militar de frecuencia.',                             4, 'Operativo', 50,  50),
    ('COM-002', 'Visor Nocturno AN/PVS-14',       'Dispositivo monocular de visión nocturna Gen III.',                                 4, 'Operativo', 40,  40),
    ('COM-003', 'Mira Telescópica',               'Mira de precisión con retícula mil-dot para fusiles de precisión.',                 4, 'Operativo', 25,  25),

    -- Vehículos y Transporte (categoria_id = 5)
    ('VEH-001', 'HMMWV (Humvee)',                 'Vehículo táctico ligero multipropósito.',                                           5, 'Operativo', 10,  10),
    ('VEH-002', 'Chevrolet Cheyenne Militarizada','Camioneta pick-up militarizada con torreta.',                                       5, 'Operativo', 15,  15),
    ('VEH-003', 'Mercedes-Benz Unimog',           'Camión mediano de transporte de tropa todo terreno.',                               5, 'Operativo', 8,   8),

    -- Equipo Médico (categoria_id = 6)
    ('MED-001', 'Botiquín Individual IFAK',       'Individual First Aid Kit para trauma de combate.',                                  6, 'Operativo', 250, 250),
    ('MED-002', 'Camilla Táctica Plegable',       'Camilla de evacuación plegable de aluminio.',                                       6, 'Operativo', 30,  30),

    -- Supervivencia y Campaña (categoria_id = 7)
    ('SUP-001', 'Ración de Combate Individual',   'RCI mexicana: paquete alimenticio para 24 horas en campaña.',                       7, 'Operativo', 500, 500),
    ('SUP-002', 'Tienda de Campaña 4 Plazas',     'Tienda militar de campaña para 4 elementos.',                                      7, 'Operativo', 60,  60);
";

/// Inicializa la tabla soldados con datos semilla cifrados a nivel de aplicación (ALE).
pub async fn inicializar_soldados_semilla(conn: &Connection) -> Result<(), String> {
    // Verificar si ya hay soldados registrados
    let mut rows = conn
        .query("SELECT COUNT(*) FROM soldados", ())
        .await
        .map_err(|e| e.to_string())?;

    let count: i64 = if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        row.get(0).map_err(|e| e.to_string())?
    } else {
        0
    };

    if count == 0 {
        let key = crate::crypto::obtener_ale_key();

        struct SoldadoSemillaRaw<'a> {
            id: i64,
            matricula: &'a str,
            nombre: &'a str,
            apellido_paterno: &'a str,
            apellido_materno: Option<&'a str>,
            rango_id: i64,
            seccion_servicio_id: i64,
            estado: &'a str,
        }

        let soldados_semilla = [
            SoldadoSemillaRaw {
                id: 1,
                matricula: "M-2309401",
                nombre: "Santiago",
                apellido_paterno: "Mendoza",
                apellido_materno: None,
                rango_id: 6,
                seccion_servicio_id: 1,
                estado: "Activo",
            },
            SoldadoSemillaRaw {
                id: 2,
                matricula: "M-2309402",
                nombre: "Mateo",
                apellido_paterno: "Guerrero",
                apellido_materno: None,
                rango_id: 4,
                seccion_servicio_id: 6,
                estado: "Activo",
            },
            SoldadoSemillaRaw {
                id: 3,
                matricula: "M-2309403",
                nombre: "Sebastián",
                apellido_paterno: "Ortega",
                apellido_materno: None,
                rango_id: 3,
                seccion_servicio_id: 2,
                estado: "Activo",
            },
        ];

        let mut sql = String::new();
        for s in &soldados_semilla {
            let blind_index = crate::crypto::calcular_blind_index(s.matricula, &key);
            let matricula_enc = crate::crypto::encriptar_pii(s.matricula, &key)?;
            let nombre_enc = crate::crypto::encriptar_pii(s.nombre, &key)?;
            let apellido_p_enc = crate::crypto::encriptar_pii(s.apellido_paterno, &key)?;
            let apellido_m_enc_str = match s.apellido_materno {
                Some(ap) => format!("'{}'", crate::crypto::encriptar_pii(ap, &key)?),
                None => "NULL".to_string(),
            };

            sql.push_str(&format!(
                "INSERT INTO soldados (id, matricula_blind_index, matricula_encriptada, nombre_encriptado, apellido_paterno_encriptado, apellido_materno_encriptado, rango_id, seccion_servicio_id, estado)
                 VALUES ({}, '{}', '{}', '{}', '{}', {}, {}, {}, '{}');\n",
                s.id, blind_index, matricula_enc, nombre_enc, apellido_p_enc, apellido_m_enc_str, s.rango_id, s.seccion_servicio_id, s.estado
            ));
        }

        conn.execute_batch(&sql)
            .await
            .map_err(|e| format!("Error ejecutando batch de soldados semilla: {}", e))?;

        tracing::info!("Soldados semilla inicializados exitosamente con ALE.");
    }
    Ok(())
}

pub async fn inicializar_credenciales_semilla(conn: &Connection) -> Result<(), String> {
    use argon2::{
        Argon2,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };

    // Verificar si ya hay credenciales registradas
    let mut rows = conn
        .query("SELECT COUNT(*) FROM credenciales_soldados", ())
        .await
        .map_err(|e| e.to_string())?;

    let count: i64 = if let Some(row) = rows.next().await.map_err(|e| e.to_string())? {
        row.get(0).map_err(|e| e.to_string())?
    } else {
        0
    };

    if count == 0 {
        let argon2 = Argon2::default();

        let usuarios_semilla = [
            (1, "subteniente.mendoza", "Mendoza2026!"),
            (2, "sargento.guerrero", "Guerrero2026!"),
            (3, "cabo.ortega", "Ortega2026!"),
        ];

        for (soldado_id, usuario, password_plana) in &usuarios_semilla {
            let salt = SaltString::generate(&mut OsRng);
            let password_hash = argon2
                .hash_password(password_plana.as_bytes(), &salt)
                .map_err(|e| format!("Error al hashear contraseña con Argon2id: {}", e))?
                .to_string();

            conn.execute(
                "INSERT INTO credenciales_soldados (soldado_id, usuario, password_hash) VALUES (?1, ?2, ?3)",
                (*soldado_id, *usuario, password_hash),
            )
            .await
            .map_err(|e| e.to_string())?;
        }
        tracing::info!("Credenciales semilla creadas exitosamente.");
    }

    Ok(())
}

/// Ejecuta todas las migraciones de esquema y datos semilla.
///
/// Utiliza `execute_batch` de TursoDB/libsql para ejecutar todas las
/// sentencias DDL y DML como una transacción atómica implícita:
/// si cualquier sentencia falla, toda la migración se revierte.
///
/// # Errores
/// Retorna un error en formato de cadena si alguna sentencia o inicialización falla.
pub async fn ejecutar_migraciones(conn: &Connection) -> Result<(), String> {
    tracing::info!("Ejecutando migraciones de esquema de base de datos...");

    // Activar claves foráneas (desactivadas por defecto en SQLite)
    // PRAGMA foreign_keys = ON debe ejecutarse FUERA de una transacción.
    conn.execute("PRAGMA foreign_keys = ON;", ())
        .await
        .map_err(|e| e.to_string())?;

    // Iniciar transacción explícita
    conn.execute("BEGIN TRANSACTION", ())
        .await
        .map_err(|e| format!("Error al iniciar transacción de migración: {}", e))?;

    // Crear tablas (DDL)
    if let Err(e) = conn.execute_batch(SCHEMA_SQL).await {
        let _ = conn.execute("ROLLBACK", ()).await;
        return Err(format!("Error en SCHEMA_SQL: {}", e));
    }
    tracing::info!("Esquema de tablas creado correctamente.");

    // Insertar datos semilla
    if let Err(e) = conn.execute_batch(SEED_RANGOS_SQL).await {
        let _ = conn.execute("ROLLBACK", ()).await;
        return Err(format!("Error en SEED_RANGOS_SQL: {}", e));
    }
    tracing::info!("Datos semilla de rangos militares insertados.");

    if let Err(e) = conn.execute_batch(SEED_SECCIONES_SQL).await {
        let _ = conn.execute("ROLLBACK", ()).await;
        return Err(format!("Error en SEED_SECCIONES_SQL: {}", e));
    }
    tracing::info!("Datos semilla de secciones y servicios insertados.");

    if let Err(e) = conn.execute_batch(SEED_CATEGORIAS_SQL).await {
        let _ = conn.execute("ROLLBACK", ()).await;
        return Err(format!("Error en SEED_CATEGORIAS_SQL: {}", e));
    }
    tracing::info!("Datos semilla de categorías de equipamiento insertadas.");

    if let Err(e) = conn.execute_batch(SEED_EQUIPAMIENTO_SQL).await {
        let _ = conn.execute("ROLLBACK", ()).await;
        return Err(format!("Error en SEED_EQUIPAMIENTO_SQL: {}", e));
    }
    tracing::info!("Datos semilla de equipamiento insertados.");

    if let Err(e) = inicializar_soldados_semilla(conn).await {
        let _ = conn.execute("ROLLBACK", ()).await;
        return Err(format!("Error en inicializar_soldados_semilla: {}", e));
    }
    tracing::info!("Datos semilla de soldados insertados con ALE.");

    if let Err(e) = inicializar_credenciales_semilla(conn).await {
        let _ = conn.execute("ROLLBACK", ()).await;
        return Err(format!("Error en inicializar_credenciales_semilla: {}", e));
    }

    if let Err(e) = conn.execute("COMMIT", ()).await {
        let _ = conn.execute("ROLLBACK", ()).await;
        return Err(format!(
            "Error al confirmar transacción de migración: {}",
            e
        ));
    }

    tracing::info!("Todas las migraciones ejecutadas exitosamente.");
    Ok(())
}

#[cfg(test)]
mod tests {
    /// Verificamos que las constantes SQL no estén vacías ni corrompidas.
    /// Las pruebas de integración contra TursoDB real se ejecutarán en
    /// un módulo separado con una BD en memoria.
    use super::*;

    #[test]
    fn test_schema_sql_contiene_todas_las_tablas() {
        let tablas_esperadas = [
            "rangos",
            "secciones_servicios",
            "soldados",
            "categorias_equipamiento",
            "equipamiento",
            "asignaciones_equipamiento",
        ];
        for tabla in &tablas_esperadas {
            assert!(
                SCHEMA_SQL.contains(&format!("CREATE TABLE IF NOT EXISTS {tabla}")),
                "Falta la tabla '{tabla}' en SCHEMA_SQL"
            );
        }
    }

    #[test]
    fn test_seed_rangos_contiene_15_rangos() {
        // Verificamos que hay exactamente 15 sentencias VALUES (una por rango)
        let count = SEED_RANGOS_SQL.matches("('").count();
        assert_eq!(count, 15, "Se esperan 15 rangos militares mexicanos");
    }

    #[test]
    fn test_seed_secciones_contiene_servicios_clave() {
        assert!(
            SEED_SECCIONES_SQL.contains("Materiales de Guerra"),
            "Falta la sección 'Materiales de Guerra' (crítica para ABAC)"
        );
        assert!(
            SEED_SECCIONES_SQL.contains("Infantería"),
            "Falta la sección 'Infantería'"
        );
    }

    #[test]
    fn test_seed_categorias_tiene_material_de_guerra() {
        assert!(
            SEED_CATEGORIAS_SQL.contains("Armamento"),
            "Falta la categoría 'Armamento'"
        );
        assert!(
            SEED_CATEGORIAS_SQL.contains("Municiones"),
            "Falta la categoría 'Municiones'"
        );
    }

    #[test]
    fn test_seed_equipamiento_contiene_items_nacionales() {
        assert!(
            SEED_EQUIPAMIENTO_SQL.contains("FX-05 Xiuhcóatl"),
            "Falta el fusil FX-05 Xiuhcóatl (fabricación DGIM)"
        );
        assert!(
            SEED_EQUIPAMIENTO_SQL.contains("Morelos"),
            "Falta el Fusil de Precisión Morelos (fabricación DGIM)"
        );
    }

    #[tokio::test]
    async fn test_ejecutar_migraciones_completo() {
        let db = turso::Builder::new_local(":memory:").build().await.unwrap();
        let conn = db.connect().unwrap();
        ejecutar_migraciones(&conn).await.unwrap();

        let mut rows = conn
            .query("SELECT COUNT(*) FROM soldados", ())
            .await
            .unwrap();
        let count: i64 = rows.next().await.unwrap().unwrap().get(0).unwrap();
        assert_eq!(count, 3);
    }
}
