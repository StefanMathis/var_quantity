var_quantity
============

[`VarQuantity`]: https://docs.rs/var_quantity/0.1.2/var_quantity/enum.VarQuantity.html
[`VarQuantity::Constant`]: https://docs.rs/var_quantity/0.1.2/var_quantity/enum.VarQuantity.html#variant.Constant
[`VarQuantity::Function`]: https://docs.rs/var_quantity/0.1.2/var_quantity/enum.VarQuantity.html#variant.Function
[`VarQuantity::get`]: https://docs.rs/var_quantity/0.1.2/var_quantity/enum.VarQuantity.html#method.get
[`FunctionWrapper`]: https://docs.rs/var_quantity/0.1.2/var_quantity/struct.FunctionWrapper.html
[`FunctionWrapper::call`]: https://docs.rs/var_quantity/0.1.2/var_quantity/struct.FunctionWrapper.html#method.call
[`FunctionWrapper::new`]: https://docs.rs/var_quantity/0.1.2/var_quantity/struct.FunctionWrapper.html#method.new
[`QuantityFunction`]: https://docs.rs/var_quantity/0.1.2/var_quantity/trait.QuantityFunction.html
[`QuantityFunction::call`]: https://docs.rs/var_quantity/0.1.2/var_quantity/trait.QuantityFunction.html#tymethod.call
[`filter_unary_function`]: https://docs.rs/var_quantity/0.1.2/var_quantity/fn.filter_unary_function.html
[`Polynomial`]: https://docs.rs/var_quantity/0.1.2/var_quantity/unary/struct.Polynomial.html
[`unary`]: https://docs.rs/var_quantity/0.1.2/var_quantity/unary.html
[typetag]: (https://docs.rs/typetag/latest/typetag/)
[uom]: (https://crates.io/crates/uom)

This crate is an extension of [dyn_quantity](https://crates.io/crates/dyn_quantity)
and provides an interface for defining variable quantities whose value is a
(pure) function of other quantities.

As an example, let's consider the eddy current losses in a conductive material
which are caused by sinusoidally changing magnetic fields. A simple model
could only take the magnetic flux density amplitude into account and a more
sophisticated model would also consider the field frequency. Using the
[`VarQuantity`] wrapper, both models can be used with the same interface:

```rust
use dyn_quantity::{DynQuantity, PredefUnit, Unit};
use var_quantity::{QuantityFunction, VarQuantity, FunctionWrapper};
use uom::si::{f64::{Power, MagneticFluxDensity, Frequency}, 
    power::watt, magnetic_flux_density::tesla, frequency::hertz};

// The serde annotations are just here because the doctests of this crate use
// the serde feature - they are not needed if the serde feature is disabled.

// Model 1: p = k * B^2
#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct Model1(DynQuantity<f64>);

#[typetag::serde]
impl QuantityFunction for Model1 {
    fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        let mut b = DynQuantity::new(0.0, PredefUnit::MagneticFluxDensity);
        for factor in influencing_factors.iter() {
            if b.unit == factor.unit {
                b = factor.clone();
            }
        }
        return self.0 * b.powi(2);
    }
}

// Model 2: p = k * f^2 * B^2
#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct Model2(DynQuantity<f64>);

#[typetag::serde]
impl QuantityFunction for Model2 {
    fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        let mut b = DynQuantity::new(0.0, PredefUnit::MagneticFluxDensity);
        let mut f = DynQuantity::new(0.0, PredefUnit::Frequency);
        for factor in influencing_factors.iter() {
            if b.unit == factor.unit {
                b = factor.clone();
            }
            if f.unit == factor.unit {
                f = factor.clone();
            }
        }
        return self.0 * f.powi(2) * b.powi(2);
    }
}

let k = DynQuantity::new(
    1000.0,
    Unit::from(PredefUnit::Power) / Unit::from(PredefUnit::MagneticFluxDensity).powi(2),
);
let model1: VarQuantity<Power> = VarQuantity::Function(
    FunctionWrapper::new(Box::new(Model1(k))).expect("output unit is watt"),
);

let k = DynQuantity::new(
    2.0,
    Unit::from(PredefUnit::Power)
        / Unit::from(PredefUnit::MagneticFluxDensity).powi(2)
        / Unit::from(PredefUnit::Frequency).powi(2),
);
let model2: VarQuantity<Power> = VarQuantity::Function(
    FunctionWrapper::new(Box::new(Model2(k))).expect("output unit is watt"),
);

// This function takes a variable quantity, the magnetic flux density and
// the frequency and calculates the losses
fn losses(model: &VarQuantity<Power>, b: MagneticFluxDensity, f: Frequency) -> Power {
    return model.get(&[b.into(), f.into()]);
}

let b = MagneticFluxDensity::new::<tesla>(1.2);
let f = Frequency::new::<hertz>(20.0);

assert_eq!(losses(&model1, b, f).get::<watt>(), 1440.0);
assert_eq!(losses(&model2, b, f).get::<watt>(), 1152.0);
```

The workflow to use the interface of this crate is as follows:
- Define the relation between input and output by implementing
[`QuantityFunction`] for the type representing a variable quantity (`Model1` and
`Model2` in the previous example). The implementor is responsible for selecting
the right quantities for his model from the give `influencing_factors` (for
unary functions, the crate provides [`filter_unary_function`] to simplify this)
and also for defining sensible defaults if the needed quantity is not given
(in the example above, the default flux density and frequency was defined to
zero). As explained in the
[serialization / deserialization](#serialization-and-deserialization) section,
the types must not be generic.
- Create an type instance and box it as a trait object. The trait object
approach is necessary for two reasons:
    1) Reduce generic bloat (for example, when a material type is defined using
    multiple [`VarQuantity`] for different properties, this could lead to dozens
    of generic parameters).
    2) To allow for serialization and deserialization using the [typetag] crate.
- Wrap the trait object in a [`FunctionWrapper`]. Since [`QuantityFunction`]
works with dynamic quantities, it needs to be tested whether the output from
[`QuantityFunction::call`] can be converted to the statically typed quantity `T`
using `TryFrom<DynQuantity<f64>>` (in the example, the quantity types provided
by the [uom] crate were used). This check is done in the constructor 
[`FunctionWrapper::new`] and again in [`FunctionWrapper::call`], see the
docstring of [`FunctionWrapper`].
- Wrap the [`FunctionWrapper`] in [`VarQuantity::Function`]. The purpose of this
enum is to offer an optimization for the important case of a constant quantity
via its second variant [`VarQuantity::Constant`]. Its [`VarQuantity::get`]
method either returns the constant quantity directly or forwards to
[`FunctionWrapper::call`].

# Predefined variable quantity models

Some variable quantity models are very common and therefore provided with this
crate. For example, model 1 from the introduction could also be realized
using the [`Polynomial`] struct from the [`unary`] module:

```rust
use dyn_quantity::{DynQuantity, PredefUnit, Unit};
use var_quantity::{unary::Polynomial, VarQuantity, FunctionWrapper};
use uom::si::{f64::{Power, MagneticFluxDensity, Frequency}, 
    power::watt, magnetic_flux_density::tesla, frequency::hertz};

// The input vector [a, b, c] is evaluated as ax² + bx + c. Here, b and c are
// zero, but still need to match unit-wise:
// [a] = W/T², [b] = W/T, [c] = W
// The output unit is [c] and the input unit is calculated as [c/b].
// [a] (and additional terms) can then be checked.
let a = DynQuantity::new(1000.0, Unit::from(PredefUnit::Power) / Unit::from(PredefUnit::MagneticFluxDensity).powi(2));
let b = DynQuantity::new(0.0, Unit::from(PredefUnit::Power) / Unit::from(PredefUnit::MagneticFluxDensity));
let c = DynQuantity::new(0.0, PredefUnit::Power);
let polynomial = Polynomial::new(vec![a, b, c]).expect("terms are checked during construction");

let model1: VarQuantity<Power> = VarQuantity::Function(
    FunctionWrapper::new(Box::new(polynomial)).expect("output unit is watt"),
);

// This function takes a variable quantity, the magnetic flux density and
// the frequency and calculates the losses
fn losses(model: &VarQuantity<Power>, b: MagneticFluxDensity, f: Frequency) -> Power {
    return model.get(&[b.into(), f.into()]);
}

let b = MagneticFluxDensity::new::<tesla>(1.2);
let f = Frequency::new::<hertz>(20.0);

assert_eq!(losses(&model1, b, f).get::<watt>(), 1440.0);
```

For a full list of available models, see the following modules:
- [`unary`] : Models representing unary functions (single input). 

# Serialization and deserialization

The serde integration is gated behind the `serde` feature flag.

All structs / enums in this crate implement serialization and deserialization.
See the docstrings of the individual types for details. The trait objects stored
within [`FunctionWrapper`] are handled via [typetag], which is why the the
implementors of [`QuantityFunction`] cannot be generic.

# Documentation

The full API documentation is available at
[https://docs.rs/var_quantity/0.1.2/var_quantity/](https://docs.rs/var_quantity/0.1.2/var_quantity/).