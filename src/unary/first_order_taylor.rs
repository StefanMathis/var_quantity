use dyn_quantity::{DynQuantity, Unit, UnitsNotEqual};

use crate::{QuantityFunction, filter_unary_function};

/**
TODO

First order taylor series.

`y = base_value * (1 + slope*(x - expansion_point))`

An example would be the linear approximation of the temperature dependency of the electrical resistivity:

`rho(T) = rho0 * (1+alpha*(T-T0))`

The quantity defines the physical type of the argument `x`.

Since the first order taylor series is just a linear function, a corresponding `From<FirstOrderTaylor> for FirstOrderTaylor` is defined.

Non-matching input -> base value is returned

Example
```

```
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
    TODO
    */
    pub fn new(
        base_value: DynQuantity<f64>,
        slope: DynQuantity<f64>,
        expansion_point: DynQuantity<f64>,
    ) -> Result<Self, UnitsNotEqual> {
        // Assert that the unit of base_value is equal to that of slope times
        // expansion_point -> Then the units are well-defined.
        let expected = base_value.unit;
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
    TODO
    */
    pub fn base_value(&self) -> &DynQuantity<f64> {
        return &self.base_value;
    }

    /**
    TODO
    */
    pub fn expansion_point(&self) -> &DynQuantity<f64> {
        return &self.expansion_point;
    }

    /**
    TODO
    */
    pub fn slope(&self) -> &DynQuantity<f64> {
        return &self.slope;
    }

    /**
    TODO
    */
    pub fn influencing_factor_unit(&self) -> &Unit {
        return &self.expansion_point.unit;
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
