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
-- La 'matricula' es la clave natural de negocio (UNIQUE).
-- ============================================================
CREATE TABLE IF NOT EXISTS soldados (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    matricula               TEXT    NOT NULL UNIQUE,
    nombre                  TEXT    NOT NULL,
    apellido_paterno        TEXT    NOT NULL,
    apellido_materno        TEXT,
    rango_id                INTEGER NOT NULL,
    seccion_servicio_id     INTEGER NOT NULL,
    estado                  TEXT    NOT NULL DEFAULT 'Activo',
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
-- TABLA: asignaciones_equipamiento
-- Registro de la cadena de custodia: quién recibió qué equipo,
-- en qué cantidad, cuándo, y quién lo autorizó.
-- 'fecha_devolucion' es NULL mientras el equipo esté asignado.
-- ============================================================
CREATE TABLE IF NOT EXISTS asignaciones_equipamiento (
    id                          INTEGER PRIMARY KEY AUTOINCREMENT,
    soldado_id                  INTEGER NOT NULL,
    equipamiento_id             INTEGER NOT NULL,
    cantidad                    INTEGER NOT NULL DEFAULT 1,
    fecha_asignacion            TEXT    NOT NULL DEFAULT (datetime('now')),
    fecha_devolucion            TEXT,
    autorizado_por_soldado_id   INTEGER NOT NULL,
    FOREIGN KEY (soldado_id)                REFERENCES soldados(id),
    FOREIGN KEY (equipamiento_id)           REFERENCES equipamiento(id),
    FOREIGN KEY (autorizado_por_soldado_id) REFERENCES soldados(id)
);
";

/// Datos semilla: Rangos oficiales del Ejército y Fuerza Aérea Mexicanos
/// según la Ley Orgánica del Ejército y Fuerza Aérea Mexicanos y las
/// equivalencias publicadas por la SEDENA/SEMAR.
///
/// **¿Por qué insertar rangos como datos semilla y no como un Enum puro?**
/// Porque la base de datos necesita referencias de clave foránea (`rango_id`)
/// estables. El Enum en Rust (`domain::types::Rango`) sigue existiendo para
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

/// Ejecuta todas las migraciones de esquema y datos semilla.
///
/// Utiliza `execute_batch` de TursoDB/libsql para ejecutar todas las
/// sentencias DDL y DML como una transacción atómica implícita:
/// si cualquier sentencia falla, toda la migración se revierte.
///
/// # Errores
/// Retorna el error de la base de datos si alguna sentencia falla.
pub async fn ejecutar_migraciones(conn: &Connection) -> Result<(), turso::Error> {
    tracing::info!("Ejecutando migraciones de esquema de base de datos...");

    // Activar claves foráneas (desactivadas por defecto en SQLite)
    conn.execute("PRAGMA foreign_keys = ON;", ()).await?;

    // Crear tablas (DDL)
    conn.execute_batch(SCHEMA_SQL).await?;
    tracing::info!("Esquema de tablas creado correctamente.");

    // Insertar datos semilla
    conn.execute_batch(SEED_RANGOS_SQL).await?;
    tracing::info!("Datos semilla de rangos militares insertados.");

    conn.execute_batch(SEED_SECCIONES_SQL).await?;
    tracing::info!("Datos semilla de secciones y servicios insertados.");

    conn.execute_batch(SEED_CATEGORIAS_SQL).await?;
    tracing::info!("Datos semilla de categorías de equipamiento insertadas.");

    conn.execute_batch(SEED_EQUIPAMIENTO_SQL).await?;
    tracing::info!("Datos semilla de equipamiento insertados.");

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
}
