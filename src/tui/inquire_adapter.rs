use inquire::validator::Validation;
use inquire::{Confirm, CustomUserError, InquireError, Text};

use super::render::render;
use crate::application::{Abandon, DeclarationPresenter, Prompter, en_euros};
use crate::domain::declaration::Declaration;
use crate::domain::expression::{ExpressionError, evaluate};
use crate::domain::money::Money;
use crate::domain::vat_rate::VatRate;

/// Questions posées dans le terminal via inquire.
pub struct PrompterInquire;

impl Prompter for PrompterInquire {
    fn demander_montant(&mut self, question: &str) -> Result<Money, Abandon> {
        let saisie = Text::new(question)
            .with_help_message("Ex. : 25,95 + 29,82")
            .with_validator(|saisie: &str| -> Result<Validation, CustomUserError> {
                Ok(match evaluate(saisie) {
                    Ok(_) => Validation::Valid,
                    Err(erreur) => Validation::Invalid(message_erreur(&erreur).into()),
                })
            })
            .with_formatter(&|saisie: &str| match evaluate(saisie) {
                Ok(total) => format!("{} → {}", saisie.trim(), en_euros(total)),
                Err(_) => saisie.to_string(),
            })
            .prompt()
            .map_err(abandon)?;
        // Le validateur a déjà refusé toute expression invalide.
        evaluate(&saisie).map_err(|_| Abandon)
    }

    fn confirmer(&mut self, message: &str) -> Result<bool, Abandon> {
        Confirm::new(message)
            .with_default(true)
            .with_parser(
                &|reponse: &str| match reponse.trim().to_lowercase().as_str() {
                    "o" | "oui" => Ok(true),
                    "n" | "non" => Ok(false),
                    _ => Err(()),
                },
            )
            .with_formatter(&|oui| if oui { "Oui" } else { "Non" }.to_string())
            .with_default_value_formatter(&|oui| if oui { "O/n" } else { "o/N" }.to_string())
            .with_error_message("Répondez « o » pour oui ou « n » pour non.")
            .prompt()
            .map_err(abandon)
    }
}

fn message_erreur(erreur: &ExpressionError) -> String {
    match erreur {
        ExpressionError::TermeInvalide(terme) if terme.is_empty() => {
            "Montant manquant (ex. : 25,95 + 29,82).".to_string()
        }
        ExpressionError::TermeInvalide(terme) => format!(
            "« {terme} » n'est pas un montant valide (chiffres, virgule ou point, deux décimales au plus)."
        ),
        ExpressionError::TotalNegatif => "Le total ne peut pas être négatif.".to_string(),
        ExpressionError::TotalTropGrand => {
            "Le total dépasse le plafond de 1 000 000 000,00 €.".to_string()
        }
    }
}

/// Échap et Ctrl-C sont des abandons ; les autres erreurs (terminal absent, E/S)
/// interrompent aussi la saisie.
fn abandon(erreur: InquireError) -> Abandon {
    match erreur {
        InquireError::OperationCanceled | InquireError::OperationInterrupted => Abandon,
        autre => {
            eprintln!("Erreur du terminal : {autre}");
            Abandon
        }
    }
}

/// Affiche la déclaration sur la sortie standard.
pub struct PresenterTerminal;

impl DeclarationPresenter for PresenterTerminal {
    fn presenter(&mut self, declaration: &Declaration, taux: VatRate) {
        println!("\n{}", render(declaration, taux));
    }
}
