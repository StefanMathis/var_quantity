/*!
An unary [`Exponential`] function which implements [`QuantityFunction`].
*/

use dyn_quantity::{DynQuantity, Unit, UnitsNotEqual};

#[cfg(feature = "serde")]
use dyn_quantity::serialize_quantity;

use crate::{QuantityFunction, filter_unary_function};

/**
An exponential term `amplitude * (exponent * x).exp` which is used to build an
[`Exponential`] function.
 */
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpTerm {
    /// Amplitude of the term.
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    pub amplitude: DynQuantity<f64>,
    /// Exponent of the term
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_quantity"))]
    pub exponent: DynQuantity<f64>,
}

/**
An exponential function defined as a sum of [`ExpTerm`]:

`y = ∑ a_n * e^(k_n * x)`

where `a` is [`ExpTerm::amplitude`] and `k` is [`ExpTerm::exponent`] of the
respective [`ExpTerm`].

This struct is meant to be used as a [`QuantityFunction`] trait object. The
unit of the influencing quantity is the inverse of the [`ExpTerm::exponent`]
unit and the output unit is that of [`ExpTerm::amplitude`]. This means that all
terms must have the same units for amplitude and exponent; this is checked in
the constructor [`Exponential::new`].

# Features:
This struct can be serialized / deserialized if the `serde` feature is enabled.
 */
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Exponential {
    terms: Vec<ExpTerm>,
    #[cfg_attr(feature = "serde", serde(skip))]
    output_unit: Unit,
    #[cfg_attr(feature = "serde", serde(skip))]
    influencing_factor_unit: Unit,
}

impl Exponential {
    /**
    Checks if the amplitudes / exponents of all given terms are identical.
    If this is the case, a new instance of [`Exponential`] is returned.

    # Examples

    ```
    use dyn_quantity::{DynQuantity, PredefUnit, Unit};
    use var_quantity::{QuantityFunction, unary::{ExpTerm, Exponential}};

    let term1 = ExpTerm {
        amplitude: DynQuantity::new(1.0, PredefUnit::Power),
        exponent: DynQuantity::new(1.0, PredefUnit::ElectricCurrent),
    };
    let term2 = ExpTerm {
        amplitude: DynQuantity::new(1.0, PredefUnit::Power),
        exponent: DynQuantity::new(1.0, PredefUnit::ElectricCurrent),
    };
    let term3 = ExpTerm {
        amplitude: DynQuantity::new(1.0, PredefUnit::Force),
        exponent: DynQuantity::new(1.0, PredefUnit::ElectricCurrent),
    };
    let term4 = ExpTerm {
        amplitude: DynQuantity::new(1.0, PredefUnit::Power),
        exponent: DynQuantity::new(1.0, PredefUnit::Force),
    };

    // Term 1 and 2 are compatible
    let exp = Exponential::new(vec![term1.clone(), term2.clone()]).unwrap();
    assert_eq!(exp.influencing_factor_unit(), Unit {
        second: 0,
        meter: 0,
        kelvin: 0,
        kilogram: 0,
        ampere: -1, // Inverse of the exponent
        mol: 0,
        candela: 0,
    });
    assert_eq!(exp.eval(&[]).unit, Unit::from(PredefUnit::Power));

    // Term 1 and 3 are not compatible (exponent mismatch)
    assert!(Exponential::new(vec![term1, term3]).is_err());

    // Term 2 and 4 are not compatible (amplitude mismatch)
    assert!(Exponential::new(vec![term2, term4]).is_err());
    ```
    */
    pub fn new(terms: Vec<ExpTerm>) -> Result<Self, UnitsNotEqual> {
        let influencing_factor_unit: Unit = match terms.first() {
            Some(t) => t.exponent.unit.powi(-1),
            None => Unit::default(),
        };

        // Check if all exponential terms have the same amplitude and the same
        // exponent units
        if let Some((first, second)) = terms.windows(2).find_map(|terms| {
            let term1 = terms[0];
            let term2 = terms[1];
            if term1.amplitude.unit != term2.amplitude.unit {
                return Some((term1.amplitude.unit, term2.amplitude.unit));
            }
            if term1.exponent.unit != term2.exponent.unit {
                return Some((term1.exponent.unit, term2.exponent.unit));
            }
            None
        }) {
            return Err(UnitsNotEqual(first, second));
        }
        let output_unit = terms
            .first()
            .map(|term| term.amplitude.unit)
            .unwrap_or_default();
        Ok(Self {
            terms,
            output_unit,
            influencing_factor_unit,
        })
    }

    /**
    Returns a slice of the underlying exponential terms.
    */
    pub fn terms(&self) -> &[ExpTerm] {
        &self.terms
    }

    /**
    Returns the unit of the quantity which influences the variable quantity.
    If none of the `conditions` in a [`QuantityFunction::eval`]
    matches this item, then `x` is assumed to be zero and the sum of amplitudes
    is returned.

    # Examples

    ```
    use dyn_quantity::{DynQuantity, PredefUnit};
    use var_quantity::{QuantityFunction, unary::{ExpTerm, Exponential}};
    use approxim::assert_abs_diff_eq;

    let term1 = ExpTerm {
        amplitude: DynQuantity::new(1.0, PredefUnit::Power),
        exponent: DynQuantity::new(0.5, PredefUnit::ElectricCurrent),
    };
    let term2 = ExpTerm {
        amplitude: DynQuantity::new(2.0, PredefUnit::Power),
        exponent: DynQuantity::new(3.0, PredefUnit::ElectricCurrent),
    };

    let exp = Exponential::new(vec![term1.clone(), term2.clone()]).unwrap();

    // No match of any input unit with influencing_factor_unit -> Just add the amplitudes
    assert_eq!(exp.eval(&[]).value, 3.0);

    // A match -> Use the corresponding quantity function as x
    let qt1 = DynQuantity::new(2.0, exp.influencing_factor_unit()); // <- Matches
    let qt2 = DynQuantity::new(-2.0, PredefUnit::Force);

    assert_abs_diff_eq!(exp.eval(&[qt1, qt2]).value, 809.576, epsilon = 1e-3);
    ```
    */
    pub fn influencing_factor_unit(&self) -> Unit {
        self.influencing_factor_unit
    }

    /**
    Returns the unit which will be returned from [`QuantityFunction::eval`].

    ```
    use dyn_quantity::{DynQuantity, PredefUnit, Unit};
    use var_quantity::{QuantityFunction, unary::{ExpTerm, Exponential}};

    let term1 = ExpTerm {
        amplitude: DynQuantity::new(1.0, PredefUnit::Power),
        exponent: DynQuantity::new(0.5, PredefUnit::ElectricCurrent),
    };
    let term2 = ExpTerm {
        amplitude: DynQuantity::new(2.0, PredefUnit::Power),
        exponent: DynQuantity::new(3.0, PredefUnit::ElectricCurrent),
    };

    let exp = Exponential::new(vec![term1.clone(), term2.clone()]).unwrap();
    assert_eq!(exp.output_unit(), Unit::from(PredefUnit::Power));
    ```
     */
    pub fn output_unit(&self) -> Unit {
        self.output_unit
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl QuantityFunction for Exponential {
    fn eval(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        filter_unary_function(
            conditions,
            self.influencing_factor_unit,
            |input| {
                // Units are already checked during construction - we can simply
                // calculate with the values directly here
                let sum = self
                    .terms
                    .iter()
                    .map(|t| t.amplitude.value * (t.exponent.value * input.value).exp())
                    .sum::<f64>();
                DynQuantity::new(sum, self.output_unit)
            },
            || {
                let sum = self.terms.iter().map(|t| t.amplitude.value).sum::<f64>();
                DynQuantity::new(sum, self.output_unit)
            },
        )
    }

    fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
        (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;

    use serde::de::{Deserialize, Deserializer};

    impl<'de> Deserialize<'de> for Exponential {
        fn deserialize<D>(deserializer: D) -> Result<Exponential, D::Error>
        where
            D: Deserializer<'de>,
        {
            #[derive(serde::Deserialize)]
            struct ExponentialAlias {
                terms: Vec<ExpTerm>,
            }

            let alias = ExponentialAlias::deserialize(deserializer)?;
            Self::new(alias.terms).map_err(serde::de::Error::custom)
        }
    }
}
