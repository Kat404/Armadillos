use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rango {
    Soldado,
    Cabo,
    Sargento,
    Teniente,
    #[serde(alias = "Capitán")]
    Capitan,
}

impl Rango {
    #[allow(dead_code)]
    pub fn puede_promover_a(self, nuevo: Rango) -> bool {
        matches!(
            (self, nuevo),
            (Rango::Soldado, Rango::Cabo)
                | (Rango::Cabo, Rango::Sargento)
                | (Rango::Sargento, Rango::Teniente)
                | (Rango::Teniente, Rango::Capitan)
        )
    }
}

impl fmt::Display for Rango {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Rango::Soldado => "Soldado",
            Rango::Cabo => "Cabo",
            Rango::Sargento => "Sargento",
            Rango::Teniente => "Teniente",
            Rango::Capitan => "Capitán",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Rango {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Soldado" => Ok(Rango::Soldado),
            "Cabo" => Ok(Rango::Cabo),
            "Sargento" => Ok(Rango::Sargento),
            "Teniente" => Ok(Rango::Teniente),
            "Capitán" | "Capitan" => Ok(Rango::Capitan),
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
        match s {
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
    use std::str::FromStr;

    #[test]
    fn test_rango_from_str() {
        assert_eq!(Rango::from_str("Soldado"), Ok(Rango::Soldado));
        assert_eq!(Rango::from_str("Cabo"), Ok(Rango::Cabo));
        assert_eq!(Rango::from_str("Sargento"), Ok(Rango::Sargento));
        assert_eq!(Rango::from_str("Teniente"), Ok(Rango::Teniente));
        assert_eq!(Rango::from_str("Capitán"), Ok(Rango::Capitan));
        assert_eq!(Rango::from_str("Capitan"), Ok(Rango::Capitan));
        assert!(Rango::from_str("Infiltrado").is_err());
    }

    #[test]
    fn test_rango_display() {
        assert_eq!(Rango::Soldado.to_string(), "Soldado");
        assert_eq!(Rango::Capitan.to_string(), "Capitán");
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
        assert!(EstadoServicio::from_str("Muerto").is_err());
    }

    #[test]
    fn test_rango_puede_promover_a() {
        // Transición válida directa
        assert!(Rango::Soldado.puede_promover_a(Rango::Cabo));
        assert!(Rango::Cabo.puede_promover_a(Rango::Sargento));
        assert!(Rango::Sargento.puede_promover_a(Rango::Teniente));
        assert!(Rango::Teniente.puede_promover_a(Rango::Capitan));

        // Transición inválida: saltando rangos
        assert!(!Rango::Soldado.puede_promover_a(Rango::Sargento));
        assert!(!Rango::Soldado.puede_promover_a(Rango::Capitan));

        // Transición inválida: degradación
        assert!(!Rango::Capitan.puede_promover_a(Rango::Teniente));
        assert!(!Rango::Cabo.puede_promover_a(Rango::Soldado));
    }
}
