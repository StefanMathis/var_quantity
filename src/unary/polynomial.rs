/*!
An unary [`Polynomial`] function which implements [`QuantityFunction`].
*/

use dyn_quantity::{DynQuantity, Unit, UnitsNotEqual};

use crate::{QuantityFunction, filter_unary_function};

/**
TODO

Explain coefficient order -> horner
 */
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Polynomial {
    coefficients: Vec<DynQuantity<f64>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    influencing_factor_unit: Unit,
    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    coefficients_val: Vec<f64>,
    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    default_value: DynQuantity<f64>,
}

impl Polynomial {
    /**
    TODO

    Explain coefficient order -> horner
    */
    pub fn new(coefficients: Vec<DynQuantity<f64>>) -> Result<Self, UnitsNotEqual> {
        let l = coefficients.len();
        let influencing_factor_unit: Unit = if l > 1 {
            // This code cannot panic, since at least two coefficients exist
            let last = coefficients[l - 1];
            let second_to_last = coefficients[l - 2];
            last.unit / second_to_last.unit
        } else {
            Unit::default()
        };

        /*
        The coefficient vector is [a, b, c, d]. The corresponding polynomial is
        [ax³ + bx² + cx + d] (same order as in horner::eval_polynomial)
        The last entry is hence the "default_value" to which the units of all
        other polynomial components are compared:
        d.unit == c.unit*influencing_factor_unit
        d.unit == b.unit*influencing_factor_unit²
        d.unit == a.unit*influencing_factor_unit³
        */
        let default_value = match coefficients.last() {
            Some(b) => {
                let base_unit = b.unit;
                for (exponent, c) in coefficients.iter().rev().enumerate().skip(1) {
                    let res_unit = c.unit * influencing_factor_unit.powi(exponent as i32);
                    if base_unit != res_unit {
                        return Err(UnitsNotEqual(base_unit, res_unit));
                    }
                }
                b.clone()
            }
            None => DynQuantity::new(0.0, Unit::default()),
        };

        let coefficients_val = coefficients.iter().map(|q| q.value).collect();
        return Ok(Self {
            coefficients,
            influencing_factor_unit,
            coefficients_val,
            default_value,
        });
    }

    /**
    Returns the `coefficients`.
    */
    pub fn coefficients(&self) -> &[DynQuantity<f64>] {
        return self.coefficients.as_slice();
    }

    /**
    Returns the unit of the quantity which influences the variable quantity.
    If none of the `influencing_factors` in a [`QuantityFunction::call`]
    matches this item, then `x` is assumed to be zero and the base value is
    returned.

    # Examples

    ```
    use std::str::FromStr;
    use dyn_quantity::{DynQuantity, PredefUnit, Unit};
    use var_quantity::{QuantityFunction, unary::Polynomial};

    let a = DynQuantity::new(
        2.0,
        Unit::from(PredefUnit::Power) / Unit::from(PredefUnit::MagneticFluxDensity).powi(2)
    );
    let b = DynQuantity::new(
        0.5,
        Unit::from(PredefUnit::Power) / Unit::from(PredefUnit::MagneticFluxDensity)
    );
    let c = DynQuantity::new(3.0, PredefUnit::Power);
    let poly = Polynomial::new(vec![a, b, c]).expect("terms are checked during construction");

    // No match of any input unit with influencing_factor_unit -> Base value
    assert_eq!(poly.call(&[]).value, 3.0);

    // A match -> Use the corresponding quantity function as x
    let qt1 = DynQuantity::new(3.0, poly.influencing_factor_unit()); // <- Matches
    let qt2 = DynQuantity::new(-2.0, PredefUnit::Force);

    assert_eq!(poly.call(&[qt1, qt2]).value, 22.5);
    ```
    */
    pub fn influencing_factor_unit(&self) -> Unit {
        return self.influencing_factor_unit;
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl QuantityFunction for Polynomial {
    fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        return filter_unary_function(
            influencing_factors,
            self.influencing_factor_unit,
            |input| {
                let val =
                    horner::eval_polynomial(input.value, self.coefficients_val.as_slice()).unwrap();
                return DynQuantity::new(val, self.default_value.unit);
            },
            || self.default_value,
        );
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;

    use serde::de::{Deserialize, Deserializer};

    impl<'de> Deserialize<'de> for Polynomial {
        fn deserialize<D>(deserializer: D) -> Result<Polynomial, D::Error>
        where
            D: Deserializer<'de>,
        {
            #[derive(serde::Deserialize)]
            struct PolynomialAlias {
                coefficients: Vec<DynQuantity<f64>>,
            }

            let alias = PolynomialAlias::deserialize(deserializer)?;
            Self::new(alias.coefficients).map_err(serde::de::Error::custom)
        }
    }
}

// =============================================================================

#[cfg(feature = "serde")]
#[cfg_attr(feature = "serde", typetag::serde)]
impl QuantityFunction for crate::ClampedQuantity<Polynomial> {
    fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        return self.call_clamped(influencing_factors);
    }
}
