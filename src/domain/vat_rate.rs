use std::fmt;

use super::money::Money;

/// Taux de TVA français, en points de base (20 % = 2000), avec sa ligne CA3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VatRate {
    basis_points: i64,
    ligne_ca3: &'static str,
    libelle_ca3: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TauxNonLegal(pub String);

const TAUX_LEGAUX: [VatRate; 4] = [
    VatRate {
        basis_points: 2000,
        ligne_ca3: "08",
        libelle_ca3: "Taux normal 20 %",
    },
    VatRate {
        basis_points: 1000,
        ligne_ca3: "9B",
        libelle_ca3: "Taux réduit 10 %",
    },
    VatRate {
        basis_points: 550,
        ligne_ca3: "09",
        libelle_ca3: "Taux réduit 5,5 %",
    },
    VatRate {
        basis_points: 210,
        ligne_ca3: "T6",
        libelle_ca3: "Opérations réalisées en France continentale au taux de 2,10 %",
    },
];

impl Default for VatRate {
    fn default() -> Self {
        TAUX_LEGAUX[0]
    }
}

impl VatRate {
    pub fn parse(input: &str) -> Result<Self, TauxNonLegal> {
        // « 5,5 » lu comme un montant donne 550 centièmes, soit directement les points de base.
        let points = Money::parse(input).map(Money::cents).ok();
        TAUX_LEGAUX
            .into_iter()
            .find(|taux| Some(taux.basis_points) == points)
            .ok_or_else(|| TauxNonLegal(input.to_string()))
    }

    pub fn basis_points(self) -> i64 {
        self.basis_points
    }

    /// Ligne CA3 qui porte la base et la taxe à ce taux.
    pub fn ligne_ca3(self) -> &'static str {
        self.ligne_ca3
    }

    /// Libellé de cette ligne sur le formulaire CA3.
    pub fn libelle_ca3(self) -> &'static str {
        self.libelle_ca3
    }
}

impl fmt::Display for VatRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (entier, decimales) = (self.basis_points / 100, self.basis_points % 100);
        if decimales == 0 {
            write!(f, "{entier} %")
        } else {
            let decimales = format!("{decimales:02}");
            write!(f, "{entier},{} %", decimales.trim_end_matches('0'))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vingt_pour_cent_par_defaut() {
        assert_eq!(VatRate::default().basis_points(), 2000);
    }

    #[test]
    fn accepte_les_taux_legaux_avec_virgule_ou_point() {
        assert_eq!(VatRate::parse("20").map(VatRate::basis_points), Ok(2000));
        assert_eq!(VatRate::parse("10").map(VatRate::basis_points), Ok(1000));
        assert_eq!(VatRate::parse("5,5").map(VatRate::basis_points), Ok(550));
        assert_eq!(VatRate::parse("5.5").map(VatRate::basis_points), Ok(550));
        assert_eq!(VatRate::parse("2,1").map(VatRate::basis_points), Ok(210));
        assert_eq!(VatRate::parse("2,10").map(VatRate::basis_points), Ok(210));
    }

    #[test]
    fn refuse_un_taux_non_legal() {
        assert!(VatRate::parse("15").is_err());
        assert!(VatRate::parse("abc").is_err());
    }

    #[test]
    fn s_affiche_a_la_francaise() {
        assert_eq!(VatRate::parse("5.5").unwrap().to_string(), "5,5 %");
        assert_eq!(VatRate::default().to_string(), "20 %");
    }

    #[test]
    fn chaque_taux_a_sa_ligne_ca3() {
        let ligne = |taux| VatRate::parse(taux).unwrap().ligne_ca3();
        assert_eq!(
            (ligne("20"), ligne("10"), ligne("5,5"), ligne("2,1")),
            ("08", "9B", "09", "T6")
        );
        assert_eq!(VatRate::default().ligne_ca3(), "08");
    }
}
