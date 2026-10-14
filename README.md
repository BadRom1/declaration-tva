# declaration-tva

Assistant en ligne de commande, pas à pas, qui aide un entrepreneur individuel
à remplir sa déclaration de TVA mensuelle CA3, y compris l'autoliquidation des
services achetés à l'étranger (facturés sans TVA).

## Installation

Téléchargez l'archive de votre plateforme depuis la page Releases du dépôt
(Linux x86_64, macOS Apple Silicon ou Windows x86_64), puis placez le binaire
dans le PATH. Exemple sous Linux et macOS :

```sh
tar xzf declaration-tva-v0.1.0-x86_64-unknown-linux-musl.tar.gz
sudo install declaration-tva /usr/local/bin/
```

Sous Windows, décompressez le `.zip` et placez `declaration-tva.exe` dans un
dossier du PATH. Chaque archive est accompagnée d'un fichier `.sha256`.

Le binaire n'est pas signé. Sur macOS, Gatekeeper le bloque : débloquez-le avec
`xattr -d com.apple.quarantine /usr/local/bin/declaration-tva`, en adaptant le
chemin à votre installation. Sous Windows, SmartScreen peut afficher un
avertissement : cliquez sur « Informations complémentaires » puis « Exécuter
quand même ».

Ou, avec un Rust stable récent (par exemple via [mise](https://mise.jdx.dev/),
`mise use -g rust`) :

```sh
cargo install --locked --git https://github.com/BadRom1/declaration-tva
```

### Publier une version

Modifiez `version` dans `Cargo.toml`, committez et poussez, puis créez et
poussez le tag correspondant :

```sh
git tag v0.1.0 && git push origin v0.1.0
```

La CI vérifie le code, construit les binaires et les publie dans une release
GitHub. Elle échoue si le tag ne correspond pas exactement à la version du
`Cargo.toml`, y compris pour une pré-version : le tag `v0.2.0-rc1` exige
`version = "0.2.0-rc1"`.

## Usage

```sh
declaration-tva            # taux de 20 % par défaut
declaration-tva --taux 10
# depuis les sources :
cargo run --release -- --taux 10
```

Taux acceptés : 20, 10, 5,5 et 2,1.

## Déroulé

1. Montant TTC facturé ce mois-ci.
2. Montant TTC de vos achats en France.
3. Montant total de vos achats à l'étranger (services facturés sans TVA).
4. Récapitulatif des trois totaux : « non » relance les trois questions.
5. Affichage des lignes CA3 en deux blocs : celles à saisir (A1, A3, base de la
   ligne du taux, 20) et celles que le formulaire calcule seul (taxe de la ligne
   du taux, 16, 23, 25 ou TD), à comparer avec l'écran.

Échap ou Ctrl-C abandonne la saisie (code de sortie 1).

Chaque montant accepte une suite d'additions et de soustractions, avec une
virgule ou un point décimal et deux décimales au plus, par exemple
`25,95 + 29,82 - 3`. Le total calculé est affiché après la saisie.

## Règles de calcul

- Les ventes TTC sont ramenées au HT, arrondi une seule fois à l'euro : ligne A1.
- Les achats à l'étranger forment la ligne A3.
- Chaque ligne est arrondie à l'euro, 0,50 étant arrondi vers le haut.
- La base et la TVA vont sur la ligne du taux : 08 (20 %), 9B (10 %),
  09 (5,5 %) ou T6 (2,1 %). Base = A1 + A3 ; TVA = arrondi(base × taux).
- Ligne 16 = TVA brute due (ligne du taux).
- Ligne 20 = TVA incluse dans les achats France (arrondie une seule fois à
  l'euro) + TVA autoliquidée (arrondi(A3 × taux)) ; ligne 23 = ligne 20.
- Solde : ligne TD (TVA due, 16 − 23) si 16 ≥ 23, sinon ligne 25 (crédit de
  TVA, 23 − 16).

## Hypothèses de la v1

- Un taux unique pour toutes les opérations du mois.
- Toutes les ventes sont soumises à la TVA française.
- La TVA des achats en France est portée en ligne 20 « Autres biens et
  services ». Les immobilisations (ligne 19) ne sont pas gérées.
- Montants plafonnés à 1 000 000 000,00 €.

## Avertissement

Les numéros de lignes A1, A3, 08, 9B, 09, T6, 16, 20, 23, 25 et TD correspondent au
formulaire CA3 en ligne et doivent être vérifiés à chaque évolution du
formulaire. Cet outil ne remplace pas un expert-comptable.

## Licence

MIT, voir [LICENSE](LICENSE).
