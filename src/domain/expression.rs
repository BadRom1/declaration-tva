use super::money::{Money, MoneyError, PLAFOND_CENTIMES};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionError {
    TermeInvalide(String),
    TotalNegatif,
    TotalTropGrand,
}

/// Évalue une saisie du type « 25,95 + 29,82 - 3 » en centimes.
pub fn evaluate(input: &str) -> Result<Money, ExpressionError> {
    let mut total: i64 = 0;
    let mut signe: i64 = 1;
    let mut terme = String::new();
    for c in input.chars().chain(std::iter::once('+')) {
        if c == '+' || c == '-' {
            let montant = Money::parse(&terme).map_err(|erreur| match erreur {
                MoneyError::TropGrand => ExpressionError::TotalTropGrand,
                _ => ExpressionError::TermeInvalide(terme.trim().to_string()),
            })?;
            total = total
                .checked_add(signe * montant.cents())
                .ok_or(ExpressionError::TotalTropGrand)?;
            signe = if c == '+' { 1 } else { -1 };
            terme.clear();
        } else {
            terme.push(c);
        }
    }
    if total < 0 {
        return Err(ExpressionError::TotalNegatif);
    }
    if total > PLAFOND_CENTIMES {
        return Err(ExpressionError::TotalTropGrand);
    }
    Ok(Money::from_cents(total))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::money::Money;

    #[test]
    fn additionne_deux_achats() {
        assert_eq!(evaluate("25,95 + 29,82"), Ok(Money::from_cents(5577)));
    }

    #[test]
    fn accepte_un_montant_seul_et_les_soustractions() {
        assert_eq!(evaluate("23,37"), Ok(Money::from_cents(2337)));
        assert_eq!(evaluate("100 - 12,5 + 0.5"), Ok(Money::from_cents(8800)));
    }

    #[test]
    fn additionne_sans_espaces() {
        assert_eq!(evaluate("25,95+29,82"), Ok(Money::from_cents(5577)));
    }

    #[test]
    fn seul_le_total_final_doit_etre_positif() {
        assert_eq!(evaluate("5 - 10 + 20"), Ok(Money::from_cents(1500)));
    }

    #[test]
    fn refuse_un_signe_initial_ou_plusieurs_separateurs() {
        assert!(evaluate("+5").is_err());
        assert!(evaluate("1,2,3").is_err());
    }

    #[test]
    fn refuse_une_expression_invalide() {
        assert!(evaluate("abc").is_err());
        assert!(evaluate("12 +").is_err());
        assert!(evaluate("").is_err());
        assert!(evaluate("2 * 3").is_err());
    }

    #[test]
    fn refuse_un_total_au_dessus_du_plafond() {
        assert_eq!(
            evaluate("600000000 + 600000000"),
            Err(ExpressionError::TotalTropGrand)
        );
        assert_eq!(evaluate("1000000001"), Err(ExpressionError::TotalTropGrand));
    }

    #[test]
    fn refuse_un_total_negatif() {
        assert_eq!(evaluate("5 - 10"), Err(ExpressionError::TotalNegatif));
    }
}
