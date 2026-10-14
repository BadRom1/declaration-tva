//! Règles de dépendance de l'architecture hexagonale : le domaine ne connaît
//! que lui-même, l'application ne connaît pas les adaptateurs.

use std::fs;
use std::path::{Path, PathBuf};

fn fichiers_rust(dossier: &Path) -> Vec<PathBuf> {
    let mut fichiers = Vec::new();
    for entree in fs::read_dir(dossier).expect("dossier source lisible") {
        let chemin = entree.expect("entrée lisible").path();
        if chemin.is_dir() {
            fichiers.extend(fichiers_rust(&chemin));
        } else if chemin.extension().is_some_and(|e| e == "rs") {
            fichiers.push(chemin);
        }
    }
    fichiers
}

/// Liste « fichier : motif » pour chaque motif interdit trouvé sous `src/<couche>`.
fn violations(couche: &str, interdits: &[&str]) -> Vec<String> {
    let dossier = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(couche);
    let fichiers = fichiers_rust(&dossier);
    assert!(
        !fichiers.is_empty(),
        "aucun fichier sous {}",
        dossier.display()
    );
    fichiers
        .iter()
        .flat_map(|fichier| {
            let contenu = fs::read_to_string(fichier).expect("fichier lisible");
            motifs_trouves(&contenu, interdits)
                .into_iter()
                .map(move |motif| format!("{} : contient « {motif} »", fichier.display()))
        })
        .collect()
}

/// Motifs interdits présents dans le code, hors lignes de commentaire.
fn motifs_trouves<'a>(contenu: &str, interdits: &[&'a str]) -> Vec<&'a str> {
    let code: Vec<&str> = contenu
        .lines()
        .filter(|ligne| !ligne.trim_start().starts_with("//"))
        .collect();
    interdits
        .iter()
        .copied()
        .filter(|motif| code.iter().any(|ligne| ligne.contains(motif)))
        .collect()
}

#[test]
fn un_motif_dans_un_commentaire_est_ignore() {
    let contenu = "// Rien à voir avec inquire.\n    /// Ni avec clap.\nuse std::fmt;\n";
    assert!(motifs_trouves(contenu, &["inquire", "clap"]).is_empty());
    assert_eq!(
        motifs_trouves("use inquire::Text; // inquire\n", &["inquire"]),
        ["inquire"]
    );
}

#[test]
fn le_domaine_ne_depend_ni_de_l_application_ni_des_adaptateurs() {
    let fautes = violations(
        "domain",
        &[
            "crate::application",
            "crate::tui",
            "declaration_tva::",
            "super::super::",
            "inquire",
            "clap",
        ],
    );
    assert!(fautes.is_empty(), "\n{}", fautes.join("\n"));
}

#[test]
fn l_application_ne_depend_pas_des_adaptateurs() {
    let fautes = violations(
        "application",
        &["crate::tui", "declaration_tva::", "inquire", "clap"],
    );
    assert!(fautes.is_empty(), "\n{}", fautes.join("\n"));
}
