/*!
An unary [`Linear`] function which implements [`QuantityFunction`].
*/

use dyn_quantity::{DynQuantity, Unit};

use crate::{QuantityFunction, filter_unary_function};

/**
A linear function defined via its `slope` and `base_value`:

`y = slope * x + base_value`

This struct is meant to be used as a [`QuantityFunction`] trait object. The
unit of the influencing quantity is `base_value`.unit / slope.unit`.

# Features:
This struct can be serialized / deserialized if the `serde` feature is enabled.
*/
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Linear {
    slope: DynQuantity<f64>,
    base_value: DynQuantity<f64>,
}

impl Linear {
    /**
    Creates a new instance of [`Linear`].

    ```
    use dyn_quantity::{DynQuantity, PredefUnit, Unit};
    use var_quantity::{QuantityFunction, unary::Linear};

    let lin = Linear::new(DynQuantity::new(-1.0, PredefUnit::Force), DynQuantity::new(2.0, PredefUnit::Torque));
    assert_eq!(lin.call(&[DynQuantity::new(5.0, PredefUnit::Length)]), DynQuantity::new(-3.0, PredefUnit::Torque))
    ```
     */
    pub fn new(slope: DynQuantity<f64>, base_value: DynQuantity<f64>) -> Self {
        return Self { slope, base_value };
    }

    /**
    Returns the `base_value`.
    */
    pub fn base_value(&self) -> &DynQuantity<f64> {
        return &self.base_value;
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
    matches this item, then `x` is assumed to be zero and `base_value` is
    returned.

    # Examples

    ```
    use std::str::FromStr;
    use dyn_quantity::{DynQuantity, PredefUnit, Unit};
    use var_quantity::{QuantityFunction, unary::Linear};

    let lin = Linear::new(
        DynQuantity::from_str("0.5 ohm/K").unwrap(),
        DynQuantity::from_str("1 ohm").unwrap(),
    );

    // No match of any input unit with influencing_factor_unit -> Base value
    assert_eq!(lin.call(&[]).value, 1.0);

    // A match -> Use the corresponding quantity function as x
    let qt1 = DynQuantity::new(2.0, lin.influencing_factor_unit()); // <- Matches
    let qt2 = DynQuantity::new(-2.0, PredefUnit::Force);

    assert_eq!(lin.call(&[qt1, qt2]).value, 2.0);
    ```
    */
    pub fn influencing_factor_unit(&self) -> Unit {
        return self.base_value.unit / self.slope.unit;
    }

    /**
    Returns the unit which will be returned from [`QuantityFunction::call`].

    ```
    use std::str::FromStr;
    use dyn_quantity::{DynQuantity, PredefUnit, Unit};
    use var_quantity::{QuantityFunction, unary::Linear};

    let lin = Linear::new(
        DynQuantity::from_str("0.5 ohm/K").unwrap(),
        DynQuantity::from_str("1 ohm").unwrap(),
    );

    assert_eq!(lin.output_unit(), Unit::from(PredefUnit::ElectricResistance));
    ```
     */
    pub fn output_unit(&self) -> Unit {
        return self.base_value.unit;
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl QuantityFunction for Linear {
    fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        return filter_unary_function(
            influencing_factors,
            self.influencing_factor_unit(),
            |input| {
                // Units are already checked during construction - we can simply
                // calculate with the values directly here
                DynQuantity::new(
                    self.base_value.value + self.slope.value * input.value,
                    self.base_value.unit,
                )
            },
            || self.base_value,
        );
    }
}

// =============================================================================

#[cfg(feature = "serde")]
#[cfg_attr(feature = "serde", typetag::serde)]
impl QuantityFunction for crate::ClampedQuantity<Linear> {
    fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        return self.call_clamped(influencing_factors);
    }
}
