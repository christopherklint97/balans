use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef},
    Decode, Encode, Sqlite, Type,
};
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, Neg, Sub};

/// Monetary type backed by rust_decimal. Serialized as string in JSON,
/// stored as integer ören (cents) in SQLite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Money(Decimal);

impl Money {
    pub const ZERO: Money = Money(Decimal::ZERO);

    /// Create a Money value from a Decimal. Rounds to 2 decimal places.
    pub fn new(amount: Decimal) -> Self {
        Self(amount.round_dp(2))
    }

    /// Create from ören (cents). 15000 ören = 150.00 SEK.
    pub fn from_ore(ore: i64) -> Self {
        Self(Decimal::new(ore, 2))
    }

    /// Convert to ören (cents) for database storage.
    pub fn to_ore(self) -> i64 {
        (self.0 * Decimal::ONE_HUNDRED)
            .round_dp(0)
            .to_string()
            .parse::<i64>()
            .expect("monetary value out of i64 range")
    }

    pub fn is_zero(self) -> bool {
        self.0.is_zero()
    }

    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    pub fn inner(self) -> Decimal {
        self.0
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl Add for Money {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.0 + rhs.0)
    }
}

impl Sub for Money {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.0 - rhs.0)
    }
}

impl Neg for Money {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Money::ZERO, |a, b| a + b)
    }
}

impl Serialize for Money {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{:.2}", self.0))
    }
}

impl<'de> Deserialize<'de> for Money {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let decimal = s
            .parse::<Decimal>()
            .map_err(serde::de::Error::custom)?;
        Ok(Money::new(decimal))
    }
}

// SQLx integration: store as INTEGER (ören) in SQLite

impl Type<Sqlite> for Money {
    fn type_info() -> SqliteTypeInfo {
        <i64 as Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <i64 as Type<Sqlite>>::compatible(ty)
    }
}

impl Encode<'_, Sqlite> for Money {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<SqliteArgumentValue<'_>>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let ore = self.to_ore();
        <i64 as Encode<Sqlite>>::encode(ore, buf)
    }
}

impl<'r> Decode<'r, Sqlite> for Money {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let ore = <i64 as Decode<Sqlite>>::decode(value)?;
        Ok(Money::from_ore(ore))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ore_roundtrip() {
        let m = Money::from_ore(15050);
        assert_eq!(m.to_ore(), 15050);
        assert_eq!(format!("{m}"), "150.50");
    }

    #[test]
    fn test_arithmetic() {
        let a = Money::from_ore(10000);
        let b = Money::from_ore(5000);
        assert_eq!((a + b).to_ore(), 15000);
        assert_eq!((a - b).to_ore(), 5000);
    }

    #[test]
    fn test_sum() {
        let vals = vec![Money::from_ore(100), Money::from_ore(200), Money::from_ore(300)];
        let total: Money = vals.into_iter().sum();
        assert_eq!(total.to_ore(), 600);
    }

    #[test]
    fn test_balance_check() {
        let debit = Money::from_ore(50000);
        let credit = Money::from_ore(50000);
        assert!((debit - credit).is_zero());
    }

    #[test]
    fn test_serde_roundtrip() {
        let m = Money::from_ore(123456);
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(json, "\"1234.56\"");
        let parsed: Money = serde_json::from_str(&json).unwrap();
        assert_eq!(m, parsed);
    }
}
