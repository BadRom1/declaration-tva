use super::money::Money;
use super::vat_rate::VatRate;

/// Les trois montants saisis pour le mois.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Saisies {
    pub ventes_ttc: Money,
    pub achats_france_ttc: Money,
    /// Services achetés à des assujettis non établis en France, facturés sans TVA.
    pub achats_etranger: Money,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Solde {
    /// Ligne TD : TVA due (ligne 16 − ligne 23).
    APayer(i64),
    /// Ligne 25 : crédit de TVA (ligne 23 − ligne 16).
    Credit(i64),
}

/// Qui remplit la ligne : l'utilisateur, ou le formulaire en ligne qui la calcule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Saisie {
    Manuelle,
    Automatique,
}

/// Une ligne du formulaire CA3, en euros entiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ligne {
    pub code: String,
    pub libelle: String,
    pub montant: i64,
    pub saisie: Saisie,
}

/// Ventilation de la CA3 du mois. Chaque ligne est arrondie à l'euro,
/// les totaux sont calculés sur ces montants entiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    pub taux: VatRate,
    pub a1: i64,
    pub a3: i64,
    pub base_taxable: i64,
    pub tva_brute_taux: i64,
    pub tva_autoliquidee: i64,
    pub ligne_16: i64,
    pub ligne_20: i64,
    pub ligne_23: i64,
    pub solde: Solde,
}

impl Declaration {
    pub fn compute(saisies: Saisies, taux: VatRate) -> Self {
        let a1 = hors_taxe(saisies.ventes_ttc, taux);
        let tva_france = tva_incluse(saisies.achats_france_ttc, taux);
        let a3 = saisies.achats_etranger.rounded_euros();
        let base_taxable = a1 + a3;
        let tva_brute_taux = tva_sur(base_taxable, taux);
        let tva_autoliquidee = tva_sur(a3, taux);
        let ligne_16 = tva_brute_taux;
        let ligne_20 = tva_france + tva_autoliquidee;
        let ligne_23 = ligne_20;
        let solde = if ligne_23 > ligne_16 {
            Solde::Credit(ligne_23 - ligne_16)
        } else {
            Solde::APayer(ligne_16 - ligne_23)
        };

        Declaration {
            taux,
            a1,
            a3,
            base_taxable,
            tva_brute_taux,
            tva_autoliquidee,
            ligne_16,
            ligne_20,
            ligne_23,
            solde,
        }
    }

    pub fn rien_a_autoliquider(&self) -> bool {
        self.a3 == 0
    }

    /// Les lignes du formulaire, dans son ordre ; les lignes nulles sont omises,
    /// sauf le solde qui est toujours indiqué.
    pub fn lignes(&self) -> Vec<Ligne> {
        use Saisie::{Automatique, Manuelle};
        let ligne = |code: &str, libelle: &str, montant, saisie| Ligne {
            code: code.to_string(),
            libelle: libelle.to_string(),
            montant,
            saisie,
        };
        let (numero, libelle) = (self.taux.ligne_ca3(), self.taux.libelle_ca3());
        let mut lignes: Vec<Ligne> = [
            ligne("A1", "Ventes, prestations de services", self.a1, Manuelle),
            ligne(
                "A3",
                "Achats de prestations de services réalisés auprès d'un assujetti \
                 non établi en France (art. 283-2 du CGI)",
                self.a3,
                Manuelle,
            ),
            ligne(
                &format!("{numero} base"),
                &format!("{libelle} : base hors taxe"),
                self.base_taxable,
                Manuelle,
            ),
            ligne(
                &format!("{numero} TVA"),
                &format!("{libelle} : taxe due"),
                self.tva_brute_taux,
                Automatique,
            ),
            ligne("16", "Total de TVA brute due", self.ligne_16, Automatique),
            ligne("20", "Autres biens et services", self.ligne_20, Manuelle),
            ligne("23", "Total TVA déductible", self.ligne_23, Automatique),
        ]
        .into_iter()
        .filter(|l| l.montant != 0)
        .collect();
        lignes.push(match self.solde {
            Solde::APayer(montant) => {
                ligne("TD", "TVA due (ligne 16 − ligne 23)", montant, Automatique)
            }
            Solde::Credit(montant) => ligne(
                "25",
                "Crédit de TVA (ligne 23 − ligne 16)",
                montant,
                Automatique,
            ),
        });
        lignes
    }
}

/// TTC × 10000 / (10000 + taux), arrondi une seule fois à l'euro (demi vers le haut).
fn hors_taxe(ttc: Money, taux: VatRate) -> i64 {
    euros_arrondis(ttc.cents() * 10_000, 10_000 + taux.basis_points())
}

/// TTC × taux / (10000 + taux), arrondi une seule fois à l'euro (demi vers le haut).
fn tva_incluse(ttc: Money, taux: VatRate) -> i64 {
    euros_arrondis(
        ttc.cents() * taux.basis_points(),
        10_000 + taux.basis_points(),
    )
}

/// Centimes × points de base ÷ diviseur en points de base, arrondi à l'euro.
fn euros_arrondis(dividende: i64, diviseur: i64) -> i64 {
    let diviseur = diviseur * 100;
    (dividende * 2 + diviseur) / (2 * diviseur)
}

/// TVA sur une base en euros entiers, arrondie à l'euro (demi vers le haut).
fn tva_sur(base_euros: i64, taux: VatRate) -> i64 {
    (base_euros * taux.basis_points() + 5_000) / 10_000
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::money::{Money, PLAFOND_CENTIMES};
    use crate::domain::vat_rate::VatRate;

    fn declarer(ventes: i64, france: i64, etranger: i64) -> Declaration {
        Declaration::compute(
            Saisies {
                ventes_ttc: Money::from_cents(ventes),
                achats_france_ttc: Money::from_cents(france),
                achats_etranger: Money::from_cents(etranger),
            },
            VatRate::default(),
        )
    }

    #[test]
    fn cas_1_railway_seul() {
        let d = declarer(0, 0, 2337);
        assert_eq!((d.a3, d.tva_autoliquidee, d.ligne_20), (23, 5, 5));
        assert_eq!(d.solde, Solde::APayer(0));
    }

    #[test]
    fn cas_2_railway_et_github() {
        let d = declarer(0, 0, 2595 + 2982);
        assert_eq!((d.a3, d.tva_autoliquidee), (56, 11));
    }

    #[test]
    fn cas_3_cinquante_centimes_arrondis_au_dessus() {
        let d = declarer(0, 0, 1020 + 1030);
        assert_eq!((d.a3, d.tva_autoliquidee), (21, 4));
    }

    #[test]
    fn les_ventes_ttc_sont_ramenees_au_ht() {
        assert_eq!(declarer(10000, 0, 0).a1, 83);
    }

    #[test]
    fn le_ht_est_arrondi_une_seule_fois() {
        let d = Declaration::compute(
            Saisies {
                ventes_ttc: Money::from_cents(10497),
                achats_france_ttc: Money::ZERO,
                achats_etranger: Money::ZERO,
            },
            VatRate::parse("5,5").unwrap(),
        );
        // HT exact 99,4976… : le double arrondi (99,50 puis 100) donnait 100.
        assert_eq!(d.a1, 99);
    }

    #[test]
    fn la_tva_des_achats_france_est_arrondie_une_seule_fois() {
        // TVA exacte 0,4983… : le double arrondi (0,50 puis 1) donnait 1.
        assert_eq!(declarer(0, 299, 0).ligne_20, 0);
        assert_eq!(declarer(0, 6000, 5577).ligne_20, 10 + 11);
    }

    #[test]
    fn mois_complet_ventile_chaque_ligne() {
        let d = declarer(120000, 6000, 5577);
        assert_eq!(
            (d.a1, d.a3, d.base_taxable, d.tva_brute_taux),
            (1000, 56, 1056, 211)
        );
        assert_eq!((d.ligne_16, d.ligne_20, d.ligne_23), (211, 21, 21));
        assert_eq!(d.solde, Solde::APayer(190));
    }

    #[test]
    fn une_tva_a_cinquante_centimes_est_arrondie_au_dessus() {
        assert_eq!(tva_sur(15, VatRate::parse("10").unwrap()), 2);
    }

    #[test]
    fn le_credit_figure_en_ligne_25_meme_sans_a1_ni_a3() {
        let d = declarer(0, 60000, 0);
        assert_eq!((d.a1, d.a3), (0, 0));
        let lignes = d.lignes();
        let codes: Vec<_> = lignes.iter().map(|l| l.code.as_str()).collect();
        assert_eq!(codes, ["20", "23", "25"]);
        assert_eq!(lignes.last().map(|l| l.montant), Some(100));
    }

    #[test]
    fn calcule_sans_debordement_au_plafond() {
        let d = declarer(PLAFOND_CENTIMES, PLAFOND_CENTIMES, PLAFOND_CENTIMES);
        assert_eq!(d.a3, 1_000_000_000);
    }

    #[test]
    fn separe_les_lignes_a_saisir_de_celles_calculees_par_le_formulaire() {
        let codes = |d: Declaration, saisie: Saisie| -> Vec<String> {
            d.lignes()
                .into_iter()
                .filter(|l| l.saisie == saisie)
                .map(|l| l.code)
                .collect()
        };
        let mois = || declarer(120000, 6000, 5577);
        assert_eq!(
            codes(mois(), Saisie::Manuelle),
            ["A1", "A3", "08 base", "20"]
        );
        assert_eq!(
            codes(mois(), Saisie::Automatique),
            ["08 TVA", "16", "23", "TD"]
        );
        assert_eq!(
            codes(declarer(0, 60000, 0), Saisie::Automatique),
            ["23", "25"]
        );
    }

    #[test]
    fn les_libelles_reprennent_le_formulaire() {
        let libelles: Vec<_> = declarer(120000, 6000, 5577)
            .lignes()
            .into_iter()
            .map(|l| l.libelle)
            .collect();
        assert_eq!(
            libelles,
            [
                "Ventes, prestations de services",
                "Achats de prestations de services réalisés auprès d'un assujetti \
                 non établi en France (art. 283-2 du CGI)",
                "Taux normal 20 % : base hors taxe",
                "Taux normal 20 % : taxe due",
                "Total de TVA brute due",
                "Autres biens et services",
                "Total TVA déductible",
                "TVA due (ligne 16 − ligne 23)",
            ]
        );
    }

    #[test]
    fn le_solde_a_payer_va_en_ligne_td_et_le_credit_en_ligne_25() {
        let derniere = |d: Declaration| d.lignes().pop().map(|l| (l.code, l.libelle));
        assert_eq!(
            derniere(declarer(120000, 6000, 5577)),
            Some(("TD".into(), "TVA due (ligne 16 − ligne 23)".into()))
        );
        assert_eq!(
            derniere(declarer(0, 60000, 0)),
            Some(("25".into(), "Crédit de TVA (ligne 23 − ligne 16)".into()))
        );
    }

    #[test]
    fn cas_reel_capture_formulaire() {
        let d = declarer(300000, 1800, 2337);
        assert_eq!((d.a1, d.a3), (2500, 23));
        assert_eq!((d.base_taxable, d.tva_brute_taux), (2523, 505));
        assert_eq!((d.ligne_16, d.ligne_20, d.ligne_23), (505, 8, 8));
        assert_eq!(d.solde, Solde::APayer(497));
        assert_eq!(
            d.lignes().last().map(|l| (l.code.as_str(), l.montant)),
            Some(("TD", 497))
        );
    }

    #[test]
    fn un_deductible_superieur_au_collecte_donne_un_credit() {
        assert_eq!(declarer(0, 60000, 0).solde, Solde::Credit(100));
    }

    #[test]
    fn cas_5_sans_achat_etranger_rien_a_autoliquider() {
        let d = declarer(120000, 0, 0);
        assert!(d.rien_a_autoliquider());
        assert!(d.lignes().iter().all(|l| l.code != "A3"));
    }

    #[test]
    fn les_lignes_a_remplir_suivent_l_ordre_du_formulaire() {
        let codes: Vec<_> = declarer(120000, 6000, 5577)
            .lignes()
            .into_iter()
            .map(|l| l.code)
            .collect();
        assert_eq!(
            codes,
            ["A1", "A3", "08 base", "08 TVA", "16", "20", "23", "TD"]
        );
    }

    #[test]
    fn a_dix_pour_cent_la_base_et_la_taxe_vont_en_ligne_9b() {
        let d = Declaration::compute(
            Saisies {
                ventes_ttc: Money::from_cents(11000),
                achats_france_ttc: Money::ZERO,
                achats_etranger: Money::ZERO,
            },
            VatRate::parse("10").unwrap(),
        );
        let codes: Vec<_> = d.lignes().into_iter().map(|l| l.code).collect();
        assert!(codes.contains(&"9B base".into()), "{codes:?}");
        assert!(codes.contains(&"9B TVA".into()), "{codes:?}");
        assert!(!codes.contains(&"08 base".into()), "{codes:?}");
    }

    #[test]
    fn a_deux_virgule_un_pour_cent_la_base_et_la_taxe_vont_en_ligne_t6() {
        let d = Declaration::compute(
            Saisies {
                ventes_ttc: Money::from_cents(102100),
                achats_france_ttc: Money::ZERO,
                achats_etranger: Money::ZERO,
            },
            VatRate::parse("2,1").unwrap(),
        );
        assert_eq!((d.a1, d.tva_brute_taux), (1000, 21));
        let lignes = d.lignes();
        assert_eq!(lignes[1].code, "T6 base");
        assert_eq!(
            lignes[1].libelle,
            "Opérations réalisées en France continentale au taux de 2,10 % : base hors taxe"
        );
        assert_eq!(lignes[2].code, "T6 TVA");
    }

    #[test]
    fn applique_le_taux_choisi() {
        let d = Declaration::compute(
            Saisies {
                ventes_ttc: Money::from_cents(11000),
                achats_france_ttc: Money::ZERO,
                achats_etranger: Money::ZERO,
            },
            VatRate::parse("10").unwrap(),
        );
        assert_eq!((d.a1, d.tva_brute_taux), (100, 10));
    }
}
