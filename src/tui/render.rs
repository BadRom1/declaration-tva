use crate::domain::declaration::{Declaration, Saisie};
use crate::domain::vat_rate::VatRate;

const TITRE_A_SAISIR: &str = "À saisir dans le formulaire";
const TITRE_CALCULE: &str = "Calculé par le formulaire — vérifiez que les montants concordent";

/// Texte affiché en fin d'assistant : lignes à saisir, lignes calculées et rappels.
pub fn render(declaration: &Declaration, taux: VatRate) -> String {
    let lignes = declaration.lignes();
    let entete = ["Ligne", "Libellé", "Montant (€)"].map(String::from);
    let cellules = |saisie| {
        let mut tableau = vec![entete.clone()];
        tableau.extend(
            lignes
                .iter()
                .filter(|l| l.saisie == saisie)
                .map(|l| [l.code.clone(), l.libelle.clone(), l.montant.to_string()]),
        );
        tableau
    };
    let blocs = [
        (TITRE_A_SAISIR, cellules(Saisie::Manuelle)),
        (TITRE_CALCULE, cellules(Saisie::Automatique)),
    ];
    // Largeurs communes aux deux blocs, pour qu'ils restent alignés.
    let largeur = |colonne: usize| {
        blocs
            .iter()
            .flat_map(|(_, tableau)| tableau)
            .map(|l| l[colonne].chars().count())
            .max()
            .unwrap_or(0)
    };
    let (l0, l1, l2) = (largeur(0), largeur(1), largeur(2));

    let mut texte = format!("Déclaration CA3 — TVA au taux de {taux}\n");
    for (titre, tableau) in &blocs {
        texte += &format!("\n{titre}\n\n");
        for (i, [code, libelle, montant]) in tableau.iter().enumerate() {
            texte += &format!("{code:<l0$} | {libelle:<l1$} | {montant:>l2$}\n");
            if i == 0 {
                texte += &format!(
                    "{}-+-{}-+-{}\n",
                    "-".repeat(l0),
                    "-".repeat(l1),
                    "-".repeat(l2)
                );
            }
        }
    }
    texte.push('\n');
    if declaration.rien_a_autoliquider() {
        texte += "Rien à autoliquider ce mois-ci.\n";
    } else {
        texte += &format!(
            "La TVA autoliquidée ({} €) est déclarée en ligne {} et déduite en ligne 20 : \
             effet net nul sur la TVA à payer.\n",
            declaration.tva_autoliquidee,
            declaration.taux.ligne_ca3()
        );
    }
    texte += "\nNuméros de lignes à vérifier à chaque évolution du formulaire CA3 en ligne.\n";
    texte
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::declaration::Saisies;
    use crate::domain::money::Money;

    fn rendre(ventes: i64, france: i64, etranger: i64) -> String {
        let declaration = Declaration::compute(
            Saisies {
                ventes_ttc: Money::from_cents(ventes),
                achats_france_ttc: Money::from_cents(france),
                achats_etranger: Money::from_cents(etranger),
            },
            VatRate::default(),
        );
        render(&declaration, VatRate::default())
    }

    /// Les cellules d'une ligne du tableau, sans les espaces d'alignement.
    fn cellules(texte: &str) -> Vec<Vec<String>> {
        texte
            .lines()
            .filter(|l| l.contains(" | "))
            .map(|l| l.split(" | ").map(|c| c.trim().to_string()).collect())
            .collect()
    }

    #[test]
    fn mois_avec_autoliquidation() {
        let texte = rendre(120000, 6000, 5577);
        assert!(texte.contains("20 %"), "{texte}");
        let attendu: Vec<Vec<String>> = [
            ["Ligne", "Libellé", "Montant (€)"],
            ["A1", "Ventes, prestations de services", "1000"],
            [
                "A3",
                "Achats de prestations de services réalisés auprès d'un assujetti \
                 non établi en France (art. 283-2 du CGI)",
                "56",
            ],
            ["08 base", "Taux normal 20 % : base hors taxe", "1056"],
            ["20", "Autres biens et services", "21"],
            ["Ligne", "Libellé", "Montant (€)"],
            ["08 TVA", "Taux normal 20 % : taxe due", "211"],
            ["16", "Total de TVA brute due", "211"],
            ["23", "Total TVA déductible", "21"],
            ["TD", "TVA due (ligne 16 − ligne 23)", "190"],
        ]
        .iter()
        .map(|l| l.iter().map(|c| c.to_string()).collect())
        .collect();
        assert_eq!(cellules(&texte), attendu);
        assert!(
            texte.contains(
                "La TVA autoliquidée (11 €) est déclarée en ligne 08 et déduite en ligne 20 : \
                 effet net nul sur la TVA à payer."
            ),
            "{texte}"
        );
        assert!(!texte.contains("Rien à autoliquider"), "{texte}");
    }

    #[test]
    fn a_cinq_et_demi_pour_cent_le_rappel_cite_la_ligne_09() {
        let taux = VatRate::parse("5,5").unwrap();
        let declaration = Declaration::compute(
            Saisies {
                ventes_ttc: Money::from_cents(120000),
                achats_france_ttc: Money::ZERO,
                achats_etranger: Money::from_cents(5577),
            },
            taux,
        );
        let texte = render(&declaration, taux);
        assert!(texte.contains("déclarée en ligne 09 "), "{texte}");
        assert!(
            cellules(&texte).iter().any(|l| l[0] == "09 base"),
            "{texte}"
        );
        assert!(!texte.contains("ligne 08"), "{texte}");
        assert!(
            cellules(&texte).iter().all(|l| l[0] != "08 base"),
            "{texte}"
        );
    }

    #[test]
    fn separe_les_lignes_a_saisir_de_celles_calculees() {
        let texte = rendre(120000, 6000, 5577);
        let (saisir, calcule) = texte
            .split_once(TITRE_CALCULE)
            .expect("bloc des lignes calculées");
        let codes = |bloc: &str| -> Vec<String> {
            cellules(bloc)
                .into_iter()
                .map(|l| l[0].clone())
                .filter(|code| code != "Ligne")
                .collect()
        };
        assert!(saisir.contains(TITRE_A_SAISIR), "{texte}");
        assert_eq!(codes(saisir), ["A1", "A3", "08 base", "20"]);
        assert_eq!(codes(calcule), ["08 TVA", "16", "23", "TD"]);
    }

    #[test]
    fn le_tableau_est_aligne() {
        let texte = rendre(120000, 6000, 5577);
        let positions: Vec<Vec<usize>> = texte
            .lines()
            .filter(|l| l.contains(" | "))
            .map(|l| {
                l.char_indices()
                    .filter(|&(_, c)| c == '|')
                    .map(|(i, _)| l[..i].chars().count())
                    .collect()
            })
            .collect();
        assert!(positions.len() > 1);
        assert!(positions.iter().all(|p| *p == positions[0]), "{texte}");
    }

    #[test]
    fn mois_sans_achat_etranger() {
        let texte = rendre(120000, 0, 0);
        assert!(texte.contains("Rien à autoliquider ce mois-ci."), "{texte}");
        assert!(!texte.contains("autoliquidée"), "{texte}");
        assert!(cellules(&texte).iter().all(|l| l[0] != "A3"), "{texte}");
    }

    #[test]
    fn mois_en_credit() {
        let texte = rendre(0, 60000, 0);
        let lignes = cellules(&texte);
        assert_eq!(
            lignes.last().unwrap(),
            &["25", "Crédit de TVA (ligne 23 − ligne 16)", "100"].map(String::from)
        );
        assert!(lignes.iter().all(|l| l[0] != "TD"), "{texte}");
    }

    #[test]
    fn se_termine_par_l_avertissement_sur_les_numeros_de_lignes() {
        let texte = rendre(120000, 6000, 5577);
        assert_eq!(
            texte.trim_end().lines().last(),
            Some("Numéros de lignes à vérifier à chaque évolution du formulaire CA3 en ligne.")
        );
    }
}
