use std::{
    fmt,
    ops::{Add, Div, Mul, Sub},
    str::FromStr,
};

use cosmwasm_schema::{
    schemars::JsonSchema,
    serde::{
        de,
        ser::{self},
        Deserialize, Deserializer, Serialize,
    },
};
use cosmwasm_std::{Decimal, StdError, Uint128};

/// A helper struct to have a signed decimal
/// This allows us to keep track if the number is positive or negative
///
/// Our usecse / example:
/// In the rebalancer, when we are doing the rebalance calculation, we can either have
/// a positive number or a negetive number.
/// positive number means we need to buy the asset, while negetive menas we need to sell.
///
/// This struct makes it easier for us to do the calculation, and act upon the sign only after
/// the calculation is done, in order to know if we should buy or sell.
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, JsonSchema, Debug)]
#[schemars(crate = "::cosmwasm_schema::schemars")]
pub struct SignedDecimal(Decimal, bool);

impl From<Decimal> for SignedDecimal {
    fn from(value: Decimal) -> Self {
        Self(value, true)
    }
}

impl From<SignedDecimal> for Decimal {
    fn from(val: SignedDecimal) -> Self {
        val.0
    }
}

impl From<Uint128> for SignedDecimal {
    fn from(value: Uint128) -> Self {
        Self(Decimal::from_atomics(value, 0).unwrap(), true)
    }
}

// impl for signed decimal
impl SignedDecimal {
    pub fn new(num: Decimal, mut sign: bool) -> Self {
        if num.is_zero() {
            sign = true;
        }
        SignedDecimal(num, sign)
    }
    pub fn __add(self, add: impl Into<SignedDecimal>) -> Self {
        let add: SignedDecimal = add.into();
        let mut res = match (self.1, add.1) {
            (true, true) => {
                // ("+" + "+") = +
                Self(self.0 + add.0, true)
            }
            (false, false) => {
                // ("-" + "-") = -
                Self(self.0 + add.0, false)
            }
            (true, false) => {
                // ("+" + "-") = -/+
                if self.0 > add.0 {
                    // ("+" > "-") = +
                    Self(self.0 - add.0, true)
                } else {
                    // ("+" < "-") = -
                    Self(add.0 - self.0, false)
                }
            }
            (false, true) => {
                // ("-" + "+") = -/+
                if self.0 > add.0 {
                    // ("-" > "+") = -
                    Self(self.0 - add.0, false)
                } else {
                    // ("-" < "+") = +
                    Self(add.0 - self.0, true)
                }
            }
        };

        if res.is_zero() {
            res.1 = true;
        }
        res
    }

    pub fn __sub(self, sub: impl Into<SignedDecimal>) -> Self {
        let sub: SignedDecimal = sub.into();
        let mut res = match (self.1, sub.1) {
            (true, true) => {
                // ("+" - "+") = -/+
                if self.0 > sub.0 {
                    Self(self.0 - sub.0, true)
                } else {
                    Self(sub.0 - self.0, false)
                }
            }
            (false, false) => {
                // ("-" - "-") = -/+
                if self.0 > sub.0 {
                    Self(self.0 - sub.0, false)
                } else {
                    Self(sub.0 - self.0, true)
                }
            }
            (true, false) => {
                // ("+" - "-") = +
                Self(self.0 + sub.0, true)
            }
            (false, true) => {
                // ("-" - "+") = -
                Self(self.0 + sub.0, false)
            }
        };

        if res.is_zero() {
            res.1 = true;
        }
        res
    }

    pub fn __mul(self, mul: impl Into<SignedDecimal>) -> Self {
        let mul: SignedDecimal = mul.into();
        match (self.1, mul.1) {
            (true, true) | (false, false) => Self(self.0 * mul.0, true),
            (true, false) | (false, true) => Self(self.0 * mul.0, false),
        }
    }

    pub fn __div(self, div: impl Into<SignedDecimal>) -> Self {
        let div: SignedDecimal = div.into();
        match (self.1, div.1) {
            (true, true) | (false, false) => Self(self.0 / div.0, true),
            (true, false) | (false, true) => Self(self.0 / div.0, false),
        }
    }

    pub fn value(self) -> Decimal {
        self.0
    }

    pub fn sign(self) -> bool {
        self.1
    }

    pub fn zero() -> Self {
        Self(Decimal::zero(), true)
    }

    pub fn positive_one() -> Self {
        Self(Decimal::one(), true)
    }

    pub fn neg_one() -> Self {
        Self(Decimal::one(), false)
    }

    pub fn to_uint_floor(self) -> Uint128 {
        self.0.to_uint_floor()
    }

    pub fn to_uint_ceil(self) -> Uint128 {
        self.0.to_uint_ceil()
    }
}

impl SignedDecimal {
    pub fn is_zero(self) -> bool {
        self.0.is_zero()
    }

    /// If our number is positive
    pub fn is_pos(self) -> bool {
        self.1
    }

    /// If our number is negetive
    pub fn is_neg(self) -> bool {
        !self.1
    }
}

impl Add for SignedDecimal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        self.__add(other)
    }
}

impl Sub for SignedDecimal {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.__sub(other)
    }
}

impl Mul for SignedDecimal {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        self.__mul(other)
    }
}

impl Div for SignedDecimal {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        self.__div(other)
    }
}

impl FromStr for SignedDecimal {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (sign, str) = match s.chars().next() {
            Some('-') => (false, &s[1..]),
            Some('+') => (true, &s[1..]),
            _ => (true, s),
        };

        let decimal = Decimal::from_str(str)?;

        Ok(SignedDecimal(decimal, sign))
    }
}

/// Serializes as a decimal string
impl Serialize for SignedDecimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let sign = if self.1 { "+" } else { "-" };
        let str = format!("{}{}", sign, self.0);
        serializer.serialize_str(&str)
    }
}

/// Deserializes as a base64 string
impl<'de> Deserialize<'de> for SignedDecimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SignedDecimalVisitor)
    }
}

struct SignedDecimalVisitor;

impl<'de> de::Visitor<'de> for SignedDecimalVisitor {
    type Value = SignedDecimal;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded signed decimal")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match SignedDecimal::from_str(v) {
            Ok(d) => Ok(d),
            Err(e) => {
                Err(E::custom(format!(
                    "Error parsing signed decimal '{v}': {e}"
                )))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use cosmwasm_std::{Decimal, Uint128};

    use super::SignedDecimal;

    #[test]
    fn test_from_into() {
        let from = SignedDecimal::from(Decimal::one());
        assert_eq!(from, SignedDecimal(Decimal::one(), true));

        let from = SignedDecimal::from(Uint128::one());
        assert_eq!(from, SignedDecimal(Decimal::one(), true));

        let into: Decimal = SignedDecimal(Decimal::one(), true).into();
        assert_eq!(into, Decimal::one());
    }

    #[test]
    fn test_helpers() {
        let dec = SignedDecimal::zero();
        assert_eq!(dec, SignedDecimal(Decimal::zero(), true));
        assert!(dec.is_zero());

        let dec = SignedDecimal::positive_one();
        assert_eq!(dec, SignedDecimal(Decimal::one(), true));

        let dec = SignedDecimal::neg_one();
        assert_eq!(dec, SignedDecimal(Decimal::one(), false));

        let dec = SignedDecimal(Decimal::from_str("1.5").unwrap(), true);
        assert_eq!(dec.to_uint_floor(), Uint128::one());
        assert_eq!(dec.to_uint_ceil(), Uint128::from(2_u128));
    }

    #[test]
    fn test_parse() {
        let correct_pos = SignedDecimal(Decimal::from_str("1.5").unwrap(), true);
        let correct_neg = SignedDecimal(Decimal::from_str("1.5").unwrap(), false);

        assert_eq!(SignedDecimal::from_str("+1.5").unwrap(), correct_pos);
        assert_eq!(SignedDecimal::from_str("1.5").unwrap(), correct_pos);
        assert_eq!(SignedDecimal::from_str("-1.5").unwrap(), correct_neg);

        assert_eq!(
            SignedDecimal::from_str("-1").unwrap(),
            SignedDecimal::neg_one()
        );
        assert_eq!(
            SignedDecimal::from_str("1").unwrap(),
            SignedDecimal::positive_one()
        );

        // should err
        SignedDecimal::from_str("-f1").unwrap_err();
        SignedDecimal::from_str("-1f").unwrap_err();
        SignedDecimal::from_str("f").unwrap_err();
        SignedDecimal::from_str("1f").unwrap_err();
    }

    #[test]
    fn test_assertion() {
        let pos = SignedDecimal(Decimal::one(), true);
        assert!(pos.is_pos());

        let neg = SignedDecimal(Decimal::one(), false);
        assert!(neg.is_neg());
    }

    #[test]
    fn test_add() {
        let ten = Decimal::from_atomics(10_u128, 0).unwrap();
        let two = Decimal::from_atomics(2_u128, 0).unwrap();

        let big_positive = SignedDecimal(ten, true);
        let big_negative = SignedDecimal(ten, false);
        let small_positive = SignedDecimal(two, true);
        let small_negative = SignedDecimal(two, false);

        // "+" + "+" = "+"
        let res = big_positive + big_positive;
        assert!(res.1);

        // "-" + "-" = "-"
        let res = big_negative + big_negative;
        assert!(!res.1);

        // "+" + "-" where "+" > "-" = "+"
        let res: SignedDecimal = big_positive + small_negative;
        assert!(res.1);

        // "+" + "-" where "+" < "-" = "-"
        let res = small_positive + big_negative;
        assert!(!res.1);

        // "-" + "+" where "-" > "+" = "-"
        let res = big_negative + small_positive;
        assert!(!res.1);

        // "-" + "+" where "-" < "+" = "+"
        let res = small_negative + big_positive;
        assert!(res.1);
    }

    #[test]
    fn test_sub() {
        let ten = Decimal::from_atomics(10_u128, 0).unwrap();
        let two = Decimal::from_atomics(2_u128, 0).unwrap();

        let big_positive = SignedDecimal(ten, true);
        let big_negative = SignedDecimal(ten, false);
        let small_positive = SignedDecimal(two, true);
        let small_negative = SignedDecimal(two, false);

        // "+" - "+" where "+" > "+" = "+"
        let res = big_positive - small_positive;
        assert!(res.1);

        // "+" - "+" where "+" < "+" = "-"
        let res = small_positive - big_positive;
        assert!(!res.1);

        // "-" - "-" where "-" > "-" = "-"
        let res = big_negative - small_negative;
        assert!(!res.1);
        assert_eq!(res.0.to_uint_floor().u128(), 8);

        // "-" - "-" where "-" < "-" = "+"
        let res = small_negative - big_negative;
        assert!(res.1);
        assert_eq!(res.0.to_uint_floor().u128(), 8);

        // "+" - "-" = "+"
        let res = small_positive - big_negative;
        assert!(res.1);

        // "-" - "+" = "-"
        let res = small_negative - big_positive;
        assert!(!res.1);
    }

    #[test]
    fn test_mul() {
        let ten = Decimal::from_atomics(10_u128, 0).unwrap();

        let positive = SignedDecimal(ten, true);
        let negative = SignedDecimal(ten, false);

        // "+" * "+" = "+"
        let res = positive * positive;
        assert!(res.1);

        // "-" * "-" = "+"
        let res = negative * negative;
        assert!(res.1);

        // "+" * "-" = "-"
        let res = positive * negative;
        assert!(!res.1);

        // "-" * "+" = "-"
        let res = negative * positive;
        assert!(!res.1);
    }

    #[test]
    fn test_div() {
        let ten = Decimal::from_atomics(10_u128, 0).unwrap();

        let positive = SignedDecimal(ten, true);
        let negative = SignedDecimal(ten, false);

        // "+" * "+" = "+"
        let res = positive / positive;
        assert!(res.1);

        // "-" * "-" = "+"
        let res = negative / negative;
        assert!(res.1);

        // "+" * "-" = "-"
        let res = positive / negative;
        assert!(!res.1);

        // "-" * "+" = "-"
        let res = negative / positive;
        assert!(!res.1);
    }

    #[test]
    fn test_verify_0_always_pos() {
        let one = SignedDecimal::positive_one();
        let neg_one = SignedDecimal::neg_one();

        let res = SignedDecimal::zero();
        assert!(res.is_pos());

        let res = SignedDecimal::new(Decimal::zero(), false);
        assert!(res.is_pos());

        let res = one - one;
        assert!(res.is_pos());

        let res = neg_one - neg_one;
        assert!(res.is_pos());

        let res = neg_one + one;
        assert!(res.is_pos());

        let res = one + neg_one;
        assert!(res.is_pos());
    }
}
