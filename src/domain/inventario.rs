#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Categoría de equipamiento militar, mapeado a la tabla `categorias_equipamiento`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CategoriaEquipamiento {
    Armamento,
    Municiones,
    EquipoTacticoIndividual,
    ComunicacionesYOptica,
    VehiculosYTransporte,
    EquipoMedico,
    SupervivenciaYCampana,
}

impl CategoriaEquipamiento {
    /// Determina si la categoría clasifica como material de guerra sensible (requiere control bélico estricto).
    pub fn es_material_de_guerra(self) -> bool {
        matches!(
            self,
            CategoriaEquipamiento::Armamento | CategoriaEquipamiento::Municiones
        )
    }
}

impl fmt::Display for CategoriaEquipamiento {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CategoriaEquipamiento::Armamento => "Armamento",
            CategoriaEquipamiento::Municiones => "Municiones",
            CategoriaEquipamiento::EquipoTacticoIndividual => "Equipo Táctico Individual",
            CategoriaEquipamiento::ComunicacionesYOptica => "Comunicaciones y Óptica",
            CategoriaEquipamiento::VehiculosYTransporte => "Vehículos y Transporte",
            CategoriaEquipamiento::EquipoMedico => "Equipo Médico",
            CategoriaEquipamiento::SupervivenciaYCampana => "Supervivencia y Campaña",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for CategoriaEquipamiento {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "Armamento" => Ok(CategoriaEquipamiento::Armamento),
            "Municiones" => Ok(CategoriaEquipamiento::Municiones),
            "Equipo Táctico Individual"
            | "Equipo Tactico Individual"
            | "EquipoTacticoIndividual" => Ok(CategoriaEquipamiento::EquipoTacticoIndividual),
            "Comunicaciones y Óptica" | "Comunicaciones y Optica" | "ComunicacionesYOptica" => {
                Ok(CategoriaEquipamiento::ComunicacionesYOptica)
            }
            "Vehículos y Transporte" | "Vehiculos y Transporte" | "VehiculosYTransporte" => {
                Ok(CategoriaEquipamiento::VehiculosYTransporte)
            }
            "Equipo Médico" | "Equipo Medico" | "EquipoMedico" => {
                Ok(CategoriaEquipamiento::EquipoMedico)
            }
            "Supervivencia y Campaña" | "Supervivencia y Campana" | "SupervivenciaYCampana" => {
                Ok(CategoriaEquipamiento::SupervivenciaYCampana)
            }
            _ => Err(format!("Categoría de equipamiento inválida: {}", s)),
        }
    }
}

/// Estado de conservación física de un ítem de equipamiento.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EstadoConservacion {
    Operativo,
    EnMantenimiento,
    DeBaja,
}

impl fmt::Display for EstadoConservacion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EstadoConservacion::Operativo => "Operativo",
            EstadoConservacion::EnMantenimiento => "En Mantenimiento",
            EstadoConservacion::DeBaja => "De Baja",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for EstadoConservacion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "Operativo" => Ok(EstadoConservacion::Operativo),
            "En Mantenimiento" | "EnMantenimiento" => Ok(EstadoConservacion::EnMantenimiento),
            "De Baja" | "DeBaja" => Ok(EstadoConservacion::DeBaja),
            _ => Err(format!("Estado de conservación inválido: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_categoria_es_material_de_guerra() {
        assert!(CategoriaEquipamiento::Armamento.es_material_de_guerra());
        assert!(CategoriaEquipamiento::Municiones.es_material_de_guerra());
        assert!(!CategoriaEquipamiento::EquipoTacticoIndividual.es_material_de_guerra());
        assert!(!CategoriaEquipamiento::EquipoMedico.es_material_de_guerra());
    }

    #[test]
    fn test_categoria_display_y_from_str() {
        assert_eq!(
            CategoriaEquipamiento::ComunicacionesYOptica.to_string(),
            "Comunicaciones y Óptica"
        );
        assert_eq!(
            CategoriaEquipamiento::from_str("Comunicaciones y Óptica"),
            Ok(CategoriaEquipamiento::ComunicacionesYOptica)
        );
        assert_eq!(
            CategoriaEquipamiento::from_str("Equipo Tactico Individual"),
            Ok(CategoriaEquipamiento::EquipoTacticoIndividual)
        );
        assert!(CategoriaEquipamiento::from_str("Invalido").is_err());
    }

    #[test]
    fn test_estado_conservacion_display_y_from_str() {
        assert_eq!(EstadoConservacion::Operativo.to_string(), "Operativo");
        assert_eq!(
            EstadoConservacion::from_str("En Mantenimiento"),
            Ok(EstadoConservacion::EnMantenimiento)
        );
        assert!(EstadoConservacion::from_str("Deteriorado").is_err());
    }
}
