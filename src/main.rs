use std::process::ExitCode;

use clap::Parser;
use declaration_tva::application::run_assistant;
use declaration_tva::domain::vat_rate::VatRate;
use declaration_tva::tui::inquire_adapter::{PresenterTerminal, PrompterInquire};

/// Assistant de remplissage de la déclaration de TVA mensuelle CA3.
#[derive(Parser)]
#[command(version)]
struct Arguments {
    /// Taux de TVA appliqué à toutes les opérations (20, 10, 5,5 ou 2,1).
    #[arg(long, default_value = "20", value_parser = parse_taux)]
    taux: VatRate,
}

fn parse_taux(saisie: &str) -> Result<VatRate, String> {
    VatRate::parse(saisie).map_err(|_| {
        format!("« {saisie} » n'est pas un taux de TVA légal ; taux acceptés : 20, 10, 5,5 ou 2,1.")
    })
}

fn main() -> ExitCode {
    let arguments = Arguments::parse();
    match run_assistant(&mut PrompterInquire, &mut PresenterTerminal, arguments.taux) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => {
            eprintln!("Saisie abandonnée.");
            ExitCode::from(1)
        }
    }
}
