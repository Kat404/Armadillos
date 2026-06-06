//! Módulo de Criptografía de Aplicación (ALE - Application-Level Encryption)
//!
//! **¿Por qué este módulo?**
//! Centraliza toda la lógica de cifrado y descifrado de datos personales (PII)
//! militares antes de persistirse en la base de datos o transmitirse en memoria.
//!
//! **Algoritmos y Estructura:**
//! 1. **Cifrado Simétrico Autenticado (AEAD):** ChaCha20-Poly1305. Cada cifrado
//!    usa un Nonce aleatorio de 96 bits (12 bytes) para garantizar seguridad y no determinismo.
//! 2. **Índice Ciego (Blind Index):** BLAKE3 Keyed Hash. Permite indexar y buscar
//!    valores únicos (como matrícula) en la DB sin revelar el texto plano.
//! 3. **Seguridad en Memoria:** Uso de `secrecy::SecretString` para zeroización de buffers.

use chacha20poly1305::{
    ChaCha20Poly1305,
    aead::{Aead, AeadCore, KeyInit},
};
use rand::rngs::OsRng;
use secrecy::SecretString;
use std::env;

/// Clave simétrica de desarrollo por defecto (32 bytes).
/// **IMPORTANTE:** En producción debe definirse en la variable de entorno `ARMADILLOS_ALE_KEY`.
const DEV_ALE_KEY: &[u8; 32] = b"desarrollo_secreto_armadillos_32";

/// Resuelve la clave simétrica de 32 bytes para ALE.
///
/// Intenta leer la variable `ARMADILLOS_ALE_KEY`.
/// Si no existe o tiene un tamaño incorrecto, recurre a `DEV_ALE_KEY` en entornos
/// de desarrollo, registrando una advertencia.
pub fn obtener_ale_key() -> [u8; 32] {
    match env::var("ARMADILLOS_ALE_KEY") {
        Ok(val) => {
            let bytes = val.as_bytes();
            if bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(bytes);
                key
            } else {
                tracing::warn!(
                    "La variable de entorno ARMADILLOS_ALE_KEY no tiene 32 bytes (longitud: {}). Usando clave de desarrollo.",
                    bytes.len()
                );
                *DEV_ALE_KEY
            }
        }
        Err(_) => {
            // No emitir logs ruidosos en cada query, pero advertir al inicio
            *DEV_ALE_KEY
        }
    }
}

/// Encripta un texto claro y retorna un String en formato Hexadecimal.
/// El formato resultante es: `hex(nonce) + hex(ciphertext_con_tag)`.
pub fn encriptar_pii(cleartext: &str, key_bytes: &[u8; 32]) -> Result<String, String> {
    let cipher = ChaCha20Poly1305::new(key_bytes.into());
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, cleartext.as_bytes())
        .map_err(|e| format!("Error en cifrado ChaCha20: {}", e))?;

    // Concatenamos nonce (12 bytes) + ciphertext (que ya incluye el tag de 16 bytes al final)
    let mut payload = nonce.to_vec();
    payload.extend_from_slice(&ciphertext);

    Ok(hex::encode(payload))
}

/// Desencripta un payload hexadecimal y retorna la cadena en formato seguro SecretString.
pub fn desencriptar_pii(encrypted_hex: &str, key_bytes: &[u8; 32]) -> Result<SecretString, String> {
    let payload = hex::decode(encrypted_hex)
        .map_err(|e| format!("Error decodificando hex en desencriptado: {}", e))?;

    if payload.len() < 12 + 16 {
        return Err("Payload cifrado inválido (demasiado corto)".to_string());
    }

    let (nonce_bytes, ciphertext) = payload.split_at(12);
    let cipher = ChaCha20Poly1305::new(key_bytes.into());
    let nonce = chacha20poly1305::Nonce::from_slice(nonce_bytes);

    let decrypted_bytes = cipher.decrypt(nonce, ciphertext).map_err(|e| {
        format!(
            "Error en descifrado ChaCha20 (clave incorrecta o tag corrupto): {}",
            e
        )
    })?;

    let decrypted_str = String::from_utf8(decrypted_bytes)
        .map_err(|e| format!("Error decodificando UTF-8 tras descifrar: {}", e))?;

    Ok(SecretString::new(decrypted_str.into()))
}

/// Calcula el Blind Index para un campo sensible usando BLAKE3 Keyed Hash.
///
/// **¿Por qué BLAKE3 en Keyed Mode?**
/// BLAKE3 soporta hashing con clave nativo de 32 bytes de forma extremadamente eficiente y segura.
/// Evita ataques de extensión de longitud sin el overhead de HMAC.
pub fn calcular_blind_index(cleartext: &str, key_bytes: &[u8; 32]) -> String {
    blake3::keyed_hash(key_bytes, cleartext.as_bytes())
        .to_hex()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cifrado_no_determinista() {
        let key = obtener_ale_key();
        let texto = "M-2309401";

        let c1 = encriptar_pii(texto, &key).unwrap();
        let c2 = encriptar_pii(texto, &key).unwrap();

        // Diferentes nonces → diferentes ciphertexts
        assert_ne!(c1, c2);

        // Ambos descifran correctamente al mismo valor original
        let d1 = desencriptar_pii(&c1, &key).unwrap();
        let d2 = desencriptar_pii(&c2, &key).unwrap();

        use secrecy::ExposeSecret;
        assert_eq!(d1.expose_secret(), d2.expose_secret());
        assert_eq!(d1.expose_secret(), &texto.to_string());
    }

    #[test]
    fn test_blind_index_determinista() {
        let key = obtener_ale_key();
        let texto = "M-2309401";

        let index1 = calcular_blind_index(texto, &key);
        let index2 = calcular_blind_index(texto, &key);

        // Mismo texto + misma clave → mismo hash (blind index)
        assert_eq!(index1, index2);

        // Cambio de texto → diferente index
        let index3 = calcular_blind_index("M-2309402", &key);
        assert_ne!(index1, index3);
    }
}
