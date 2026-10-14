/// Montant positif ou nul, stocké en centimes entiers pour éviter les flottants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Money {
    cents: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyError {
    Invalide(String),
    Negatif,
    TropGrand,
}

/// 1 000 000 000,00 € : avec ce plafond, les multiplications de `declaration.rs` restent sûres en i64.
pub const PLAFOND_CENTIMES: i64 = 100_000_000_000;

impl Money {
    pub const ZERO: Money = Money { cents: 0 };

    pub fn from_cents(cents: i64) -> Self {
        Money { cents }
    }

    /// Accepte « 23 », « 23,5 », « 23.37 » : au plus deux décimales, jamais de signe.
    pub fn parse(input: &str) -> Result<Self, MoneyError> {
        let invalide = || MoneyError::Invalide(input.to_string());
        let text = input.trim();
        let (euros, decimales) = match text.split_once([',', '.']) {
            Some((e, d)) => (e, d),
            None => (text, ""),
        };
        let chiffres = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
        if text.starts_with('-') {
            return Err(MoneyError::Negatif);
        }
        if !chiffres(euros)
            || decimales.len() > 2
            || (text.len() != euros.len() && !chiffres(decimales))
        {
            return Err(invalide());
        }
        let euros: i64 = euros.parse().map_err(|_| MoneyError::TropGrand)?;
        let centimes: i64 = format!("{decimales:0<2}").parse().map_err(|_| invalide())?;
        euros
            .checked_mul(100)
            .and_then(|c| c.checked_add(centimes))
            .filter(|&c| c <= PLAFOND_CENTIMES)
            .map(Money::from_cents)
            .ok_or(MoneyError::TropGrand)
    }

    pub fn cents(self) -> i64 {
        self.cents
    }

    /// Arrondi à l'euro le plus proche, 0,50 arrondi à l'euro supérieur.
    pub fn rounded_euros(self) -> i64 {
        (self.cents + 50) / 100
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_un_montant_avec_virgule_en_centimes() {
        assert_eq!(Money::parse("23,37"), Ok(Money::from_cents(2337)));
    }

    #[test]
    fn parse_un_montant_avec_point_et_sans_decimales() {
        assert_eq!(Money::parse("23.5"), Ok(Money::from_cents(2350)));
        assert_eq!(Money::parse("23"), Ok(Money::from_cents(2300)));
    }

    #[test]
    fn refuse_un_montant_non_numerique_ou_negatif_ou_trop_precis() {
        assert!(Money::parse("abc").is_err());
        assert!(Money::parse("-3").is_err());
        assert!(Money::parse("1,234").is_err());
        assert!(Money::parse("").is_err());
    }

    #[test]
    fn refuse_un_separateur_sans_chiffres_de_part_et_d_autre() {
        assert!(Money::parse("23,").is_err());
        assert!(Money::parse(".5").is_err());
        assert!(Money::parse("1,").is_err());
    }

    #[test]
    fn accepte_zero_avec_decimales() {
        assert_eq!(Money::parse("0,00"), Ok(Money::ZERO));
    }

    #[test]
    fn refuse_un_montant_au_dessus_du_plafond() {
        assert_eq!(
            Money::parse("1000000000,00"),
            Ok(Money::from_cents(100_000_000_000))
        );
        assert_eq!(Money::parse("1000000000,01"), Err(MoneyError::TropGrand));
        assert_eq!(
            Money::parse("99999999999999999999"),
            Err(MoneyError::TropGrand)
        );
    }

    #[test]
    fn arrondit_a_l_euro_le_plus_proche() {
        assert_eq!(Money::from_cents(2337).rounded_euros(), 23);
        assert_eq!(Money::from_cents(5577).rounded_euros(), 56);
    }

    #[test]
    fn arrondit_cinquante_centimes_a_l_euro_superieur() {
        assert_eq!(Money::from_cents(2050).rounded_euros(), 21);
        assert_eq!(Money::from_cents(2049).rounded_euros(), 20);
    }
}
