#!/usr/bin/env nu

# Script de Respaldo Cifrado Asimétrico para la Base de Datos de Armadillos.
# Utiliza `sqlite3` para un backup consistente en caliente y `age` para cifrado asimétrico.

def main [
    --db-path: string = "armadillos.db"     # Ruta a la base de datos original
    --backup-dir: string = "backups"       # Directorio donde se guardarán los respaldos
    --key-file: string = ""                 # Ruta al archivo de llave privada/pública de age. Si no se provee, se generará uno automáticamente.
] {
    # 1. Validaciones previas
    if not ($db_path | path exists) {
        print $"Error: La base de datos ($db_path) no existe."
        return { success: false, error: "Database not found" }
    }

    # Asegurar que el directorio de backups existe
    mkdir $backup_dir

    # 2. Gestión de Llaves de age
    mut key_path = $key_file
    if ($key_path | is-empty) {
        $key_path = $"($backup_dir)/age_backup_key.txt"
        if not ($key_path | path exists) {
            print $"Clave de cifrado no provista. Generando nueva clave age en ($key_path)..."
            if (which age-keygen | is-empty) {
                print "Error: El comando `age-keygen` no está instalado en el sistema."
                return { success: false, error: "age-keygen missing" }
            }
            age-keygen -o $key_path
        }
    }

    # Leer la clave pública de age desde el archivo de llaves
    if not ($key_path | path exists) {
        print $"Error: El archivo de llaves ($key_path) no existe."
        return { success: false, error: "Key file not found" }
    }

    # Extraer la clave pública del archivo de clave generado/existente
    # age-keygen genera un archivo que contiene "# public key: age1..."
    let key_content = (open $key_path | lines)
    let public_key = ($key_content 
        | where {|line| $line | str starts-with "# public key: " } 
        | first 
        | str replace "# public key: " "")

    if ($public_key | is-empty) {
        print "Error: No se pudo extraer la clave pública del archivo."
        return { success: false, error: "Public key extraction failed" }
    }

    # 3. Preparar nombres de archivos
    let timestamp = (date now | format date "%Y%m%d_%H%M%S")
    let temp_backup = $"($backup_dir)/temp_($timestamp).db"
    let encrypted_backup = $"($backup_dir)/backup_($timestamp).db.age"

    print $"[1/3] Realizando backup en caliente de ($db_path) usando VACUUM INTO..."
    
    # Ejecutar VACUUM INTO para obtener una copia 100% consistente libre de bloqueos WAL
    let sql_res = (sqlite3 $db_path $"VACUUM INTO '($temp_backup)';" err> /dev/null)
    if not ($temp_backup | path exists) {
        print "Error: Falló la creación del backup temporal de SQLite."
        return { success: false, error: "SQLite VACUUM failed" }
    }

    let original_size = (ls $temp_backup | get 0.size)

    print $"[2/3] Cifrando backup con age utilizando la clave pública ($public_key)..."
    if (which age | is-empty) {
        print "Error: El comando `age` no está instalado en el sistema."
        # Limpieza del archivo temporal para evitar fuga de PII
        rm -f $temp_backup
        return { success: false, error: "age command missing" }
    }

    # Cifrar el archivo temporal
    let encrypt_res = (age -r $public_key -o $encrypted_backup $temp_backup)
    
    # Eliminar el backup temporal inmediatamente (Zeroización de PII local)
    rm -f $temp_backup

    if not ($encrypted_backup | path exists) {
        print "Error: Falló el cifrado del archivo de backup."
        return { success: false, error: "Encryption failed" }
    }

    let encrypted_size = (ls $encrypted_backup | get 0.size)

    print $"[3/3] Respaldo cifrado exitosamente en ($encrypted_backup)"

    # 4. Registro estructurado (Log)
    let log_file = $"($backup_dir)/backups_log.json"
    let log_entry = {
        timestamp: (date now | format date "%Y-%m-%d %H:%M:%S"),
        database: $db_path,
        backup_file: $encrypted_backup,
        original_size_bytes: ($original_size | into int),
        encrypted_size_bytes: ($encrypted_size | into int),
        public_key_used: $public_key,
        success: true
    }

    let logs = (if ($log_file | path exists) {
        try { open $log_file } catch { [] }
    } else {
        []
    })
    
    let logs = ($logs | append $log_entry)
    $logs | save --force $log_file

    print $"Log de respaldo registrado en ($log_file)"
    return $log_entry
}
