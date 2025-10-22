use dyn_quantity::{DynQuantity, Unit};

use crate::{QuantityFunction, filter_unary_function};

/**
TODO
*/
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Linear {
    slope: DynQuantity<f64>,
    base_value: DynQuantity<f64>,
}

impl Linear {
    /**
    TODO

    The linear Siso-function is defined by the following formula:

    `y = slope * x + base_value`
    ```

    // let linear = Linear::new(2.0, 1.0);
    // assert_eq!(linear.eval(0.0), 1.0);
    // assert_eq!(linear.eval(1.0), 3.0);
    ```
     */
    pub fn new(slope: DynQuantity<f64>, base_value: DynQuantity<f64>) -> Self {
        return Self { slope, base_value };
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
    pub fn slope(&self) -> &DynQuantity<f64> {
        return &self.slope;
    }

    /**
    TODO
    */
    pub fn influencing_factor_unit(&self) -> Unit {
        return self.base_value.unit / self.slope.unit;
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
