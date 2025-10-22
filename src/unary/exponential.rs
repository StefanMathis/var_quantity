use dyn_quantity::{DynQuantity, Unit, UnitsNotEqual};

use crate::{QuantityFunction, filter_unary_function};

/**
TODO
 */
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpTerm {
    pub amplitude: DynQuantity<f64>,
    pub exponent: DynQuantity<f64>,
}

/**
TODO
 */
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Exponential {
    terms: Vec<ExpTerm>,
    #[cfg_attr(feature = "serde", serde(skip))]
    amplitude_unit: Unit,
    #[cfg_attr(feature = "serde", serde(skip))]
    influencing_factor_unit: Unit,
}

impl Exponential {
    /**
    TODO
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
            return None;
        }) {
            return Err(UnitsNotEqual(first, second));
        }
        let amplitude_unit = terms
            .get(0)
            .map(|term| term.amplitude.unit)
            .unwrap_or(Default::default());
        return Ok(Self {
            terms,
            amplitude_unit,
            influencing_factor_unit,
        });
    }

    /**
    TODO
    */
    pub fn terms(&self) -> &[ExpTerm] {
        return &self.terms;
    }

    /**
    TODO
    */
    pub fn influencing_factor_unit(&self) -> Unit {
        return self.influencing_factor_unit;
    }
}

#[cfg_attr(feature = "serde", typetag::serde)]
impl QuantityFunction for Exponential {
    fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        return filter_unary_function(
            influencing_factors,
            self.influencing_factor_unit,
            |input| {
                // Units are already checked during construction - we can simply
                // calculate with the values directly here
                let sum = self
                    .terms
                    .iter()
                    .map(|t| t.amplitude.value * (t.exponent.value * input.value).exp())
                    .sum::<f64>();
                return DynQuantity::new(sum, self.amplitude_unit);
            },
            || {
                let sum = self.terms.iter().map(|t| t.amplitude.value).sum::<f64>();
                return DynQuantity::new(sum, self.amplitude_unit);
            },
        );
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
