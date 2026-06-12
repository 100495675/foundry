use bincode::Options;

/// Errores posibles durante la deserialización de una matriz `.matrix`.
#[derive(Debug)]
pub enum MatrixError {
    CorruptedHeader,
    InvalidMagic,
    UnsupportedVersion(u16),
    EndiannessConflict,
    TruncatedPayload,
    DeserializationFailed,
}

/// Marcador de endianness del artefacto.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum Endianness {
    Little = 0x01,
    Big = 0x02,
}

impl Endianness {
    #[inline(always)]
    const fn current() -> Self {
        #[cfg(target_endian = "little")]
        {
            Endianness::Little
        }
        #[cfg(target_endian = "big")]
        {
            Endianness::Big
        }
    }
}

/// Tamaño fijo de la cabecera `.matrix` en bytes (Actualizado a 39 por TYPE_HASH).
const HEADER_SIZE: usize = 39;

/// Extrae de forma segura y veloz los datos binarios contiguos mapeados en el ejecutable.
///
/// Realiza validación completa de cabecera física (magia, endianness, versión, truncado)
/// antes de delegar la deserialización a Bincode con la endianness nativa del hardware.
pub fn cast_from_matrix<T>(matrix_bytes: &[u8]) -> Result<T, MatrixError>
where
    T: serde::de::DeserializeOwned,
{
    // Validación de longitud mínima de cabecera
    if matrix_bytes.len() < HEADER_SIZE {
        return Err(MatrixError::CorruptedHeader);
    }

    // Validación de bytes mágicos
    if &matrix_bytes[0..4] != b"MATR" {
        return Err(MatrixError::InvalidMagic);
    }

    // Validación de marca de endianness
    let matrix_endian = match matrix_bytes[4] {
        0x01 => Endianness::Little,
        0x02 => Endianness::Big,
        _ => return Err(MatrixError::CorruptedHeader),
    };

    // Validar cross-endian accidental en runtime
    if matrix_endian != Endianness::current() {
        return Err(MatrixError::EndiannessConflict);
    }

    // Al garantizar la coincidencia de endianness, los bytes se leen de forma nativa
    let version = u16::from_ne_bytes([matrix_bytes[5], matrix_bytes[6]]);
    if version != 1 {
        return Err(MatrixError::UnsupportedVersion(version));
    }

    let read_u64_native = |offset: usize| -> u64 {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&matrix_bytes[offset..offset + 8]);
        u64::from_ne_bytes(buf)
    };

    // Ajustado el índice de lectura del payload de 23 a 31 por el desplazamiento del hash
    let payload_len = read_u64_native(31) as usize;
    let payload_end = HEADER_SIZE + payload_len;

    // Validar que el payload no esté truncado
    if matrix_bytes.len() < payload_end {
        return Err(MatrixError::TruncatedPayload);
    }

    let payload = &matrix_bytes[HEADER_SIZE..payload_end];

    bincode_options()
        .deserialize(payload)
        .map_err(|_| MatrixError::DeserializationFailed)
}

pub fn bincode_options() -> impl bincode::Options {
    bincode::options()
        .with_little_endian()
        .with_varint_encoding()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct EstructuraPrueba {
        texto: String,
        numero: u32,
    }

    #[test]
    fn validar_reconstitucion_local_posix() {
        let objeto = EstructuraPrueba {
            texto: "Datos_Congelados_Nativos".to_string(),
            numero: 1337,
        };

        use bincode::Options;
        let payload = bincode::options()
            .with_little_endian()
            .serialize(&objeto)
            .unwrap();

        // Esculpimos la nueva cabecera física definitiva de 39 bytes
        let mut matriz_artesanal = Vec::new();
        matriz_artesanal.extend_from_slice(b"MATR"); // MAGIC (4B)
        matriz_artesanal.push(0x01); // ENDIAN MARK: LE (1B)
        matriz_artesanal.extend_from_slice(&1u16.to_le_bytes()); // VERSION (2B)
        matriz_artesanal.extend_from_slice(&11111u64.to_le_bytes()); // AST HASH (8B)
        matriz_artesanal.extend_from_slice(&22222u64.to_le_bytes()); // DEP HASH (8B)
        matriz_artesanal.extend_from_slice(&33333u64.to_le_bytes()); // TYPE HASH (8B) [NUEVO]
        matriz_artesanal.extend_from_slice(&(payload.len() as u64).to_le_bytes()); // PAYLOAD LEN (8B)

        matriz_artesanal.extend_from_slice(&payload);

        // Llamada 100% segura sin bloques unsafe
        let resultado: Result<EstructuraPrueba, MatrixError> = cast_from_matrix(&matriz_artesanal);

        assert!(resultado.is_ok(), "El runtime rechazó la matriz local");
        let objeto_fundido = resultado.unwrap();

        assert_eq!(objeto_fundido.texto, "Datos_Congelados_Nativos");
        assert_eq!(objeto_fundido.numero, 1337);
    }
}