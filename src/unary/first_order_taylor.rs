/*!
An unary [`FirstOrderTaylor`] function which implements [`QuantityFunction`].
*/

use dyn_quantity::{DynQuantity, Unit, UnitsNotEqual};

use crate::{QuantityFunction, filter_unary_function};

/**
A first order taylor series function defined as:

`y = base_value * (1 + slope*(x - expansion_point))`

This struct is meant to be used as a [`QuantityFunction`] trait object. The
unit of the influencing quantity is `expansion_point`.unit`.

# Examples

A real-life example is the linear approximation of the temperature dependency of
the electrical resistivity:

`rho(T) = rho0 * (1+alpha*(T-T0))`

```
use std::str::FromStr;
use dyn_quantity::DynQuantity;
use var_quantity::{QuantityFunction, unary::FirstOrderTaylor};

// Matching units
let fot = FirstOrderTaylor::new(
    DynQuantity::from_str("2 ohm * m").unwrap(), // rho0
    DynQuantity::from_str("0.5 / K").unwrap(), // alpha
    DynQuantity::from_str("30 K").unwrap() // T0
).expect("units match");

assert_eq!(fot.call(&[ DynQuantity::from_str("60 K").unwrap()]).value, 32.0);
```

# Features:
This struct can be serialized / deserialized if the `serde` feature is enabled.
*/
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FirstOrderTaylor {
    base_value: DynQuantity<f64>,
    slope: DynQuantity<f64>,
    expansion_point: DynQuantity<f64>,
}

impl FirstOrderTaylor {
    /**
    Checks if `base_value.unit = slope.unit * expansion_point.unit`.
    If this is the case, a new instance of [`FirstOrderTaylor`] is returned.

    # Examples

    ```
    use std::str::FromStr;
    use dyn_quantity::DynQuantity;
    use var_quantity::{QuantityFunction, unary::FirstOrderTaylor};

    // Matching units
    assert!(FirstOrderTaylor::new(
        DynQuantity::from_str("1 ohm").unwrap(),
        DynQuantity::from_str("0.5 / K").unwrap(),
        DynQuantity::from_str("30 K").unwrap()
    ).is_ok());

    // Mismatched units
    assert!(FirstOrderTaylor::new(
        DynQuantity::from_str("1 ohm").unwrap(),
        DynQuantity::from_str("0.5 A").unwrap(),
        DynQuantity::from_str("30 K").unwrap()
    ).is_err());
    ```
    */
    pub fn new(
        base_value: DynQuantity<f64>,
        slope: DynQuantity<f64>,
        expansion_point: DynQuantity<f64>,
    ) -> Result<Self, UnitsNotEqual> {
        // Assert that the product of expansion_point unit and slope unit is
        // unitless.
        let expected = Unit::default();
        let found = expansion_point.unit * slope.unit;
        if expected == found {
            return Ok(Self {
                base_value,
                slope,
                expansion_point,
            });
        } else {
            return Err(UnitsNotEqual(expected, found));
        }
    }

    /**
    Returns the `base_value`.
    */
    pub fn base_value(&self) -> &DynQuantity<f64> {
        return &self.base_value;
    }

    /**
    Returns the `expansion_point`.
    */
    pub fn expansion_point(&self) -> &DynQuantity<f64> {
        return &self.expansion_point;
    }

    /**
    Returns the `slope`.
    */
    pub fn slope(&self) -> &DynQuantity<f64> {
        return &self.slope;
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
    use var_quantity::{QuantityFunction, unary::FirstOrderTaylor};

    let fot = FirstOrderTaylor::new(
        DynQuantity::from_str("1 ohm").unwrap(),
        DynQuantity::from_str("0.5 / K").unwrap(),
        DynQuantity::from_str("30 K").unwrap()
    ).expect("matching units");

    // No match of any input unit with influencing_factor_unit -> Base value
    assert_eq!(fot.call(&[]).value, 1.0);

    // A match -> Use the corresponding quantity function as x
    let qt1 = DynQuantity::new(32.0, fot.influencing_factor_unit()); // <- Matches
    let qt2 = DynQuantity::new(-2.0, PredefUnit::Force);

    assert_eq!(fot.call(&[qt1, qt2]).value, 2.0);
    ```
    */
    pub fn influencing_factor_unit(&self) -> Unit {
        return self.expansion_point.unit;
    }

    /**
    Returns the unit which will be returned from [`QuantityFunction::call`].

    ```
    use std::str::FromStr;
    use dyn_quantity::{DynQuantity, PredefUnit, Unit};
    use var_quantity::{QuantityFunction, unary::FirstOrderTaylor};

    // Matching units
    let fot = FirstOrderTaylor::new(
        DynQuantity::from_str("1 ohm").unwrap(),
        DynQuantity::from_str("0.5 / K").unwrap(),
        DynQuantity::from_str("30 K").unwrap()
    ).expect("matching units");

    assert_eq!(fot.output_unit(), Unit::from(PredefUnit::ElectricResistance));
    ```
     */
    pub fn output_unit(&self) -> Unit {
        return self.base_value.unit;
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl QuantityFunction for FirstOrderTaylor {
    fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        return filter_unary_function(
            influencing_factors,
            self.expansion_point.unit,
            |input| {
                // Units are already checked during construction - we can simply
                // calculate with the values directly here
                self.base_value
                    * (1.0 + self.slope.value * (input.value - self.expansion_point.value))
            },
            || self.base_value,
        );
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;

    use serde::de::{Deserialize, Deserializer};

    impl<'de> Deserialize<'de> for FirstOrderTaylor {
        fn deserialize<D>(deserializer: D) -> Result<FirstOrderTaylor, D::Error>
        where
            D: Deserializer<'de>,
        {
            #[derive(serde::Deserialize)]
            struct FirstOrderTaylorAlias {
                base_value: DynQuantity<f64>,
                expansion_point: DynQuantity<f64>,
                slope: DynQuantity<f64>,
            }

            let alias = FirstOrderTaylorAlias::deserialize(deserializer)?;
            Self::new(alias.base_value, alias.slope, alias.expansion_point)
                .map_err(serde::de::Error::custom)
        }
    }
}

// =============================================================================

#[cfg(feature = "serde")]
#[cfg_attr(feature = "serde", typetag::serde)]
impl QuantityFunction for crate::ClampedQuantity<FirstOrderTaylor> {
    fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        return self.call_clamped(influencing_factors);
    }
}
