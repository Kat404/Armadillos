#![allow(dead_code)]
use crate::domain::tipos_militares::{CategoriaRango, Rango};

/// Contexto de acceso que encapsula los atributos del usuario
/// necesarios para evaluar permisos ABAC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextoAcceso {
    pub rango: Rango,
    pub es_servicio_belico: bool, // Viene de secciones_servicios.es_servicio_belico
}

impl ContextoAcceso {
    /// Evalúa si el usuario puede realizar operaciones CRUD sobre
    /// equipamiento clasificado como material de guerra.
    ///
    /// Regla:
    /// 1. Oficiales, Jefes y Generales (orden >= 6) → Sí, siempre.
    /// 2. Clases (Sargento 2do / 1ro) del Servicio de Materiales de Guerra (es_servicio_belico = true) → Sí.
    /// 3. Cualquier otro caso (Cabos sin importar sección, Soldados, personal fuera de materiales de guerra) → No.
    pub fn puede_administrar_inventario_belico(&self) -> bool {
        match self.rango.categoria() {
            CategoriaRango::Oficiales | CategoriaRango::Jefes | CategoriaRango::Generales => true,
            CategoriaRango::Clases => {
                // Cabo es una clase pero no se le autoriza la administración bélica autónoma (solo Sargento Segundo y Primero)
                if self.rango == Rango::Cabo {
                    false
                } else {
                    self.es_servicio_belico
                }
            }
            CategoriaRango::Tropa => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tipos_militares::Rango;

    #[test]
    fn test_abac_oficiales_siempre_acceden() {
        let oficiales = [
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
        for &r in oficiales.iter() {
            let ctx_si = ContextoAcceso {
                rango: r,
                es_servicio_belico: true,
            };
            let ctx_no = ContextoAcceso {
                rango: r,
                es_servicio_belico: false,
            };
            assert!(ctx_si.puede_administrar_inventario_belico());
            assert!(ctx_no.puede_administrar_inventario_belico());
        }
    }

    #[test]
    fn test_abac_tropa_nunca_accede() {
        let tropa = [Rango::Soldado, Rango::SoldadoDePrimera];
        for &r in tropa.iter() {
            let ctx_si = ContextoAcceso {
                rango: r,
                es_servicio_belico: true,
            };
            let ctx_no = ContextoAcceso {
                rango: r,
                es_servicio_belico: false,
            };
            assert!(!ctx_si.puede_administrar_inventario_belico());
            assert!(!ctx_no.puede_administrar_inventario_belico());
        }
    }

    #[test]
    fn test_abac_cabo_nunca_accede() {
        let ctx_si = ContextoAcceso {
            rango: Rango::Cabo,
            es_servicio_belico: true,
        };
        let ctx_no = ContextoAcceso {
            rango: Rango::Cabo,
            es_servicio_belico: false,
        };
        assert!(!ctx_si.puede_administrar_inventario_belico());
        assert!(!ctx_no.puede_administrar_inventario_belico());
    }

    #[test]
    fn test_abac_sargento_con_servicio_belico() {
        let sargentos = [Rango::SargentoSegundo, Rango::SargentoPrimero];
        for &r in sargentos.iter() {
            let ctx = ContextoAcceso {
                rango: r,
                es_servicio_belico: true,
            };
            assert!(ctx.puede_administrar_inventario_belico());
        }
    }

    #[test]
    fn test_abac_sargento_sin_servicio_belico() {
        let sargentos = [Rango::SargentoSegundo, Rango::SargentoPrimero];
        for &r in sargentos.iter() {
            let ctx = ContextoAcceso {
                rango: r,
                es_servicio_belico: false,
            };
            assert!(!ctx.puede_administrar_inventario_belico());
        }
    }
}
