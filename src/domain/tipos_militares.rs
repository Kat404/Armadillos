#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Los 15 rangos oficiales del Ejército y Fuerza Aérea Mexicanos,
/// ordenados de menor a mayor jerarquía según la Ley Orgánica del
/// Ejército y Fuerza Aérea Mexicanos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rango {
    Soldado,
    SoldadoDePrimera,
    Cabo,
    SargentoSegundo,
    SargentoPrimero,
    Subteniente,
    Teniente,
    CapitanSegundo,
    CapitanPrimero,
    Mayor,
    TenienteCoronel,
    Coronel,
    GeneralBrigadier,
    GeneralDeBrigada,
    GeneralDeDivision,
}

/// Categoría orgánica según la escala jerárquica mexicana.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CategoriaRango {
    Tropa,     // Soldado, Soldado de Primera
    Clases,    // Cabo, Sargento Segundo, Sargento Primero
    Oficiales, // Subteniente → Capitán Primero
    Jefes,     // Mayor → Coronel
    Generales, // General Brigadier → General de División
}

impl Rango {
    /// Devuelve el orden jerárquico del rango (1-15),
    /// correspondiente a la columna `orden_jerarquico` de la BD.
    pub fn orden_jerarquico(self) -> u8 {
        match self {
            Rango::Soldado => 1,
            Rango::SoldadoDePrimera => 2,
            Rango::Cabo => 3,
            Rango::SargentoSegundo => 4,
            Rango::SargentoPrimero => 5,
            Rango::Subteniente => 6,
            Rango::Teniente => 7,
            Rango::CapitanSegundo => 8,
            Rango::CapitanPrimero => 9,
            Rango::Mayor => 10,
            Rango::TenienteCoronel => 11,
            Rango::Coronel => 12,
            Rango::GeneralBrigadier => 13,
            Rango::GeneralDeBrigada => 14,
            Rango::GeneralDeDivision => 15,
        }
    }

    /// Devuelve la categoría orgánica del rango.
    pub fn categoria(self) -> CategoriaRango {
        match self {
            Rango::Soldado | Rango::SoldadoDePrimera => CategoriaRango::Tropa,
            Rango::Cabo | Rango::SargentoSegundo | Rango::SargentoPrimero => CategoriaRango::Clases,
            Rango::Subteniente
            | Rango::Teniente
            | Rango::CapitanSegundo
            | Rango::CapitanPrimero => CategoriaRango::Oficiales,
            Rango::Mayor | Rango::TenienteCoronel | Rango::Coronel => CategoriaRango::Jefes,
            Rango::GeneralBrigadier | Rango::GeneralDeBrigada | Rango::GeneralDeDivision => {
                CategoriaRango::Generales
            }
        }
    }

    /// Determina si un militar con `self` puede ser promovido a `nuevo`.
    /// Regla: Solo se permite la promoción al rango inmediatamente superior.
    pub fn puede_promover_a(self, nuevo: Rango) -> bool {
        let actual_orden = self.orden_jerarquico();
        let nuevo_orden = nuevo.orden_jerarquico();
        nuevo_orden == actual_orden + 1
    }

    /// Determina si `self` tiene mayor jerarquía que `otro`.
    pub fn supera_a(self, otro: Rango) -> bool {
        self.orden_jerarquico() > otro.orden_jerarquico()
    }

    /// Convierte el `rango_id` (de 1 a 15) de la BD al Enum.
    pub fn from_id(id: i64) -> Result<Self, String> {
        match id {
            1 => Ok(Rango::Soldado),
            2 => Ok(Rango::SoldadoDePrimera),
            3 => Ok(Rango::Cabo),
            4 => Ok(Rango::SargentoSegundo),
            5 => Ok(Rango::SargentoPrimero),
            6 => Ok(Rango::Subteniente),
            7 => Ok(Rango::Teniente),
            8 => Ok(Rango::CapitanSegundo),
            9 => Ok(Rango::CapitanPrimero),
            10 => Ok(Rango::Mayor),
            11 => Ok(Rango::TenienteCoronel),
            12 => Ok(Rango::Coronel),
            13 => Ok(Rango::GeneralBrigadier),
            14 => Ok(Rango::GeneralDeBrigada),
            15 => Ok(Rango::GeneralDeDivision),
            _ => Err(format!("ID de rango inválido: {}", id)),
        }
    }

    /// Convierte el Enum al `rango_id` correspondiente en la BD.
    pub fn to_id(self) -> i64 {
        self.orden_jerarquico() as i64
    }
}

impl fmt::Display for Rango {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Rango::Soldado => "Soldado",
            Rango::SoldadoDePrimera => "Soldado de Primera",
            Rango::Cabo => "Cabo",
            Rango::SargentoSegundo => "Sargento Segundo",
            Rango::SargentoPrimero => "Sargento Primero",
            Rango::Subteniente => "Subteniente",
            Rango::Teniente => "Teniente",
            Rango::CapitanSegundo => "Capitán Segundo",
            Rango::CapitanPrimero => "Capitán Primero",
            Rango::Mayor => "Mayor",
            Rango::TenienteCoronel => "Teniente Coronel",
            Rango::Coronel => "Coronel",
            Rango::GeneralBrigadier => "General Brigadier",
            Rango::GeneralDeBrigada => "General de Brigada",
            Rango::GeneralDeDivision => "General de División",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Rango {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "Soldado" => Ok(Rango::Soldado),
            "Soldado de Primera" | "Soldado de primera" | "SoldadoDePrimera" => {
                Ok(Rango::SoldadoDePrimera)
            }
            "Cabo" => Ok(Rango::Cabo),
            "Sargento Segundo" | "Sargento segundo" | "SargentoSegundo" => {
                Ok(Rango::SargentoSegundo)
            }
            "Sargento Primero" | "Sargento primero" | "SargentoPrimero" => {
                Ok(Rango::SargentoPrimero)
            }
            "Subteniente" => Ok(Rango::Subteniente),
            "Teniente" => Ok(Rango::Teniente),
            "Capitán Segundo" | "Capitán segundo" | "Capitan Segundo" | "Capitan segundo"
            | "CapitanSegundo" => Ok(Rango::CapitanSegundo),
            "Capitán Primero" | "Capitán primero" | "Capitan Primero" | "Capitan primero"
            | "CapitanPrimero" => Ok(Rango::CapitanPrimero),
            "Mayor" => Ok(Rango::Mayor),
            "Teniente Coronel" | "Teniente coronel" | "TenienteCoronel" => {
                Ok(Rango::TenienteCoronel)
            }
            "Coronel" => Ok(Rango::Coronel),
            "General Brigadier" | "General brigadier" | "GeneralBrigadier" => {
                Ok(Rango::GeneralBrigadier)
            }
            "General de Brigada" | "General de brigada" | "GeneralDeBrigada" => {
                Ok(Rango::GeneralDeBrigada)
            }
            "General de División"
            | "General de división"
            | "General de Division"
            | "General de division"
            | "GeneralDeDivision" => Ok(Rango::GeneralDeDivision),
            _ => Err(format!("Rango inválido: {}", s)),
        }
    }
}

impl TryFrom<String> for Rango {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Rango::from_str(&value)
    }
}

/// Estado del personal militar en activo, de licencia o retirado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EstadoServicio {
    Activo,
    Licencia,
    Retirado,
}

impl fmt::Display for EstadoServicio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EstadoServicio::Activo => "Activo",
            EstadoServicio::Licencia => "Licencia",
            EstadoServicio::Retirado => "Retirado",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for EstadoServicio {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "Activo" => Ok(EstadoServicio::Activo),
            "Licencia" => Ok(EstadoServicio::Licencia),
            "Retirado" => Ok(EstadoServicio::Retirado),
            _ => Err(format!("Estado de servicio inválido: {}", s)),
        }
    }
}

impl TryFrom<String> for EstadoServicio {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        EstadoServicio::from_str(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rango_orden_jerarquico_completo() {
        let rangos = [
            Rango::Soldado,
            Rango::SoldadoDePrimera,
            Rango::Cabo,
            Rango::SargentoSegundo,
            Rango::SargentoPrimero,
            Rango::Subteniente,
            Rango::Teniente,
            Rango::CapitanSegundo,
            Rango::CapitanPrimero,
            Rango::Mayor,
            Rango::TenienteCoronel,
            Rango::Coronel,
            Rango::GeneralBrigadier,
            Rango::GeneralDeBrigada,
            Rango::GeneralDeDivision,
        ];
        for (i, &r) in rangos.iter().enumerate() {
            assert_eq!(r.orden_jerarquico(), (i + 1) as u8);
        }
    }

    #[test]
    fn test_rango_categoria() {
        assert_eq!(Rango::Soldado.categoria(), CategoriaRango::Tropa);
        assert_eq!(Rango::SoldadoDePrimera.categoria(), CategoriaRango::Tropa);
        assert_eq!(Rango::Cabo.categoria(), CategoriaRango::Clases);
        assert_eq!(Rango::SargentoSegundo.categoria(), CategoriaRango::Clases);
        assert_eq!(Rango::SargentoPrimero.categoria(), CategoriaRango::Clases);
        assert_eq!(Rango::Subteniente.categoria(), CategoriaRango::Oficiales);
        assert_eq!(Rango::CapitanPrimero.categoria(), CategoriaRango::Oficiales);
        assert_eq!(Rango::Mayor.categoria(), CategoriaRango::Jefes);
        assert_eq!(Rango::Coronel.categoria(), CategoriaRango::Jefes);
        assert_eq!(
            Rango::GeneralBrigadier.categoria(),
            CategoriaRango::Generales
        );
        assert_eq!(
            Rango::GeneralDeDivision.categoria(),
            CategoriaRango::Generales
        );
    }

    #[test]
    fn test_rango_display_15_variantes() {
        assert_eq!(Rango::Soldado.to_string(), "Soldado");
        assert_eq!(Rango::SoldadoDePrimera.to_string(), "Soldado de Primera");
        assert_eq!(Rango::CapitanSegundo.to_string(), "Capitán Segundo");
        assert_eq!(Rango::CapitanPrimero.to_string(), "Capitán Primero");
        assert_eq!(Rango::GeneralDeDivision.to_string(), "General de División");
    }

    #[test]
    fn test_rango_from_str_15_variantes() {
        assert_eq!(Rango::from_str("Soldado"), Ok(Rango::Soldado));
        assert_eq!(
            Rango::from_str("Soldado de Primera"),
            Ok(Rango::SoldadoDePrimera)
        );
        assert_eq!(
            Rango::from_str("Capitán Segundo"),
            Ok(Rango::CapitanSegundo)
        );
        assert_eq!(
            Rango::from_str("Capitan Segundo"),
            Ok(Rango::CapitanSegundo)
        );
        assert_eq!(
            Rango::from_str("General de División"),
            Ok(Rango::GeneralDeDivision)
        );
        assert_eq!(
            Rango::from_str("General de Division"),
            Ok(Rango::GeneralDeDivision)
        );
        assert!(Rango::from_str("Rebelde").is_err());
    }

    #[test]
    fn test_rango_from_id_y_to_id_roundtrip() {
        let rangos = [
            Rango::Soldado,
            Rango::SoldadoDePrimera,
            Rango::Cabo,
            Rango::SargentoSegundo,
            Rango::SargentoPrimero,
            Rango::Subteniente,
            Rango::Teniente,
            Rango::CapitanSegundo,
            Rango::CapitanPrimero,
            Rango::Mayor,
            Rango::TenienteCoronel,
            Rango::Coronel,
            Rango::GeneralBrigadier,
            Rango::GeneralDeBrigada,
            Rango::GeneralDeDivision,
        ];
        for &r in rangos.iter() {
            let id = r.to_id();
            assert_eq!(Rango::from_id(id), Ok(r));
        }
        assert!(Rango::from_id(0).is_err());
        assert!(Rango::from_id(16).is_err());
    }

    #[test]
    fn test_rango_puede_promover_a() {
        assert!(Rango::Soldado.puede_promover_a(Rango::SoldadoDePrimera));
        assert!(Rango::SoldadoDePrimera.puede_promover_a(Rango::Cabo));
        assert!(Rango::Coronel.puede_promover_a(Rango::GeneralBrigadier));

        // Saltos no permitidos
        assert!(!Rango::Soldado.puede_promover_a(Rango::Cabo));
        // Degradación
        assert!(!Rango::Cabo.puede_promover_a(Rango::SoldadoDePrimera));
    }

    #[test]
    fn test_rango_supera_a() {
        assert!(Rango::GeneralDeDivision.supera_a(Rango::GeneralDeBrigada));
        assert!(Rango::Cabo.supera_a(Rango::SoldadoDePrimera));
        assert!(!Rango::Soldado.supera_a(Rango::Cabo));
    }

    #[test]
    fn test_rango_ord_derivado() {
        assert!(Rango::Soldado < Rango::Cabo);
        assert!(Rango::Coronel > Rango::Mayor);
        assert!(Rango::GeneralDeDivision > Rango::GeneralBrigadier);
    }

    #[test]
    fn test_estado_servicio_from_str() {
        assert_eq!(
            EstadoServicio::from_str("Activo"),
            Ok(EstadoServicio::Activo)
        );
        assert_eq!(
            EstadoServicio::from_str("Licencia"),
            Ok(EstadoServicio::Licencia)
        );
        assert_eq!(
            EstadoServicio::from_str("Retirado"),
            Ok(EstadoServicio::Retirado)
        );
        assert!(EstadoServicio::from_str("Invalido").is_err());
    }
}
