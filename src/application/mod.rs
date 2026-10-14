use crate::domain::declaration::{Declaration, Saisies};
use crate::domain::money::Money;
use crate::domain::vat_rate::VatRate;

/// L'utilisateur a quitté l'assistant (Échap ou Ctrl-C).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Abandon;

/// Port d'entrée : questions posées à l'utilisateur.
pub trait Prompter {
    /// Redemande tant que la saisie n'est pas une expression valide.
    fn demander_montant(&mut self, question: &str) -> Result<Money, Abandon>;
    fn confirmer(&mut self, message: &str) -> Result<bool, Abandon>;
}

/// Port de sortie : restitution de la déclaration calculée.
pub trait DeclarationPresenter {
    fn presenter(&mut self, declaration: &Declaration, taux: VatRate);
}

pub const QUESTION_VENTES: &str = "Montant TTC facturé ce mois-ci ?";
pub const QUESTION_ACHATS_FRANCE: &str = "Montant TTC de vos achats en France ?";
pub const QUESTION_ACHATS_ETRANGER: &str =
    "Montant total de vos achats à l'étranger (services facturés sans TVA) ?";

/// Déroulé de l'assistant : saisie, confirmation (sinon on recommence), puis présentation.
pub fn run_assistant(
    prompter: &mut impl Prompter,
    presenter: &mut impl DeclarationPresenter,
    taux: VatRate,
) -> Result<(), Abandon> {
    let saisies = loop {
        let saisies = Saisies {
            ventes_ttc: prompter.demander_montant(QUESTION_VENTES)?,
            achats_france_ttc: prompter.demander_montant(QUESTION_ACHATS_FRANCE)?,
            achats_etranger: prompter.demander_montant(QUESTION_ACHATS_ETRANGER)?,
        };
        if prompter.confirmer(&recapitulatif(&saisies))? {
            break saisies;
        }
    };
    presenter.presenter(&Declaration::compute(saisies, taux), taux);
    Ok(())
}

fn recapitulatif(saisies: &Saisies) -> String {
    format!(
        "Ventes TTC : {}, achats France TTC : {}, achats à l'étranger : {}. Est-ce correct ?",
        en_euros(saisies.ventes_ttc),
        en_euros(saisies.achats_france_ttc),
        en_euros(saisies.achats_etranger),
    )
}

/// Montant à la française : « 55,77 € ».
pub fn en_euros(montant: Money) -> String {
    let cents = montant.cents();
    format!("{},{:02} €", cents / 100, cents % 100)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    enum Reponse {
        Montant(i64),
        Confirmation(bool),
        Abandon,
    }

    #[derive(Default)]
    struct PrompterScripte {
        reponses: VecDeque<Reponse>,
        questions: Vec<String>,
        confirmations: Vec<String>,
    }

    impl PrompterScripte {
        fn avec(reponses: impl IntoIterator<Item = Reponse>) -> Self {
            PrompterScripte {
                reponses: reponses.into_iter().collect(),
                ..Default::default()
            }
        }
    }

    impl Prompter for PrompterScripte {
        fn demander_montant(&mut self, question: &str) -> Result<Money, Abandon> {
            self.questions.push(question.to_string());
            match self.reponses.pop_front() {
                Some(Reponse::Montant(c)) => Ok(Money::from_cents(c)),
                Some(Reponse::Abandon) => Err(Abandon),
                _ => panic!("montant inattendu : {question}"),
            }
        }

        fn confirmer(&mut self, message: &str) -> Result<bool, Abandon> {
            self.confirmations.push(message.to_string());
            match self.reponses.pop_front() {
                Some(Reponse::Confirmation(oui)) => Ok(oui),
                Some(Reponse::Abandon) => Err(Abandon),
                _ => panic!("confirmation inattendue : {message}"),
            }
        }
    }

    #[derive(Default)]
    struct PresenterEnregistreur {
        presentees: Vec<(Declaration, VatRate)>,
    }

    impl DeclarationPresenter for PresenterEnregistreur {
        fn presenter(&mut self, declaration: &Declaration, taux: VatRate) {
            self.presentees.push((declaration.clone(), taux));
        }
    }

    use Reponse::{Confirmation, Montant};

    fn saisies(ventes: i64, france: i64, etranger: i64) -> Saisies {
        Saisies {
            ventes_ttc: Money::from_cents(ventes),
            achats_france_ttc: Money::from_cents(france),
            achats_etranger: Money::from_cents(etranger),
        }
    }

    #[test]
    fn pose_les_trois_questions_dans_l_ordre() {
        let mut prompter =
            PrompterScripte::avec([Montant(0), Montant(0), Montant(0), Confirmation(true)]);
        run_assistant(
            &mut prompter,
            &mut PresenterEnregistreur::default(),
            VatRate::default(),
        )
        .unwrap();
        assert_eq!(
            prompter.questions,
            [
                QUESTION_VENTES,
                QUESTION_ACHATS_FRANCE,
                QUESTION_ACHATS_ETRANGER
            ]
        );
    }

    #[test]
    fn presente_la_declaration_calculee_avec_les_montants_saisis() {
        let mut prompter = PrompterScripte::avec([
            Montant(120000),
            Montant(6000),
            Montant(5577),
            Confirmation(true),
        ]);
        let mut presenter = PresenterEnregistreur::default();
        let taux = VatRate::parse("10").unwrap();
        run_assistant(&mut prompter, &mut presenter, taux).unwrap();
        assert_eq!(
            presenter.presentees,
            [(
                Declaration::compute(saisies(120000, 6000, 5577), taux),
                taux
            )]
        );
    }

    #[test]
    fn le_recapitulatif_reprend_les_trois_totaux() {
        let mut prompter = PrompterScripte::avec([
            Montant(120000),
            Montant(6000),
            Montant(5577),
            Confirmation(true),
        ]);
        run_assistant(
            &mut prompter,
            &mut PresenterEnregistreur::default(),
            VatRate::default(),
        )
        .unwrap();
        let recap = &prompter.confirmations[0];
        assert!(recap.contains("1200,00 €"), "{recap}");
        assert!(recap.contains("60,00 €"), "{recap}");
        assert!(recap.contains("55,77 €"), "{recap}");
    }

    #[test]
    fn recommence_les_questions_apres_un_refus() {
        let mut prompter = PrompterScripte::avec([
            Montant(100),
            Montant(200),
            Montant(300),
            Confirmation(false),
            Montant(120000),
            Montant(6000),
            Montant(5577),
            Confirmation(true),
        ]);
        let mut presenter = PresenterEnregistreur::default();
        run_assistant(&mut prompter, &mut presenter, VatRate::default()).unwrap();
        assert_eq!(prompter.questions.len(), 6);
        assert_eq!(prompter.questions[3], QUESTION_VENTES);
        assert_eq!(
            presenter.presentees,
            [(
                Declaration::compute(saisies(120000, 6000, 5577), VatRate::default()),
                VatRate::default()
            )]
        );
    }

    #[test]
    fn un_abandon_se_propage_sans_rien_presenter() {
        let mut prompter = PrompterScripte::avec([Montant(100), Reponse::Abandon]);
        let mut presenter = PresenterEnregistreur::default();
        let resultat = run_assistant(&mut prompter, &mut presenter, VatRate::default());
        assert_eq!(resultat, Err(Abandon));
        assert!(presenter.presentees.is_empty());
    }

    #[test]
    fn un_abandon_a_la_confirmation_ne_presente_rien() {
        let mut prompter =
            PrompterScripte::avec([Montant(0), Montant(0), Montant(0), Reponse::Abandon]);
        let mut presenter = PresenterEnregistreur::default();
        let resultat = run_assistant(&mut prompter, &mut presenter, VatRate::default());
        assert_eq!(resultat, Err(Abandon));
        assert!(presenter.presentees.is_empty());
    }

    #[test]
    fn affiche_les_montants_a_la_francaise() {
        assert_eq!(en_euros(Money::from_cents(5577)), "55,77 €");
        assert_eq!(en_euros(Money::from_cents(5)), "0,05 €");
        assert_eq!(en_euros(Money::ZERO), "0,00 €");
    }
}
