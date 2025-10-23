#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use std::marker::PhantomData;

use dyn_quantity::{DynQuantity, Unit, UnitFromType, UnitsNotEqual};

use num::Complex;
#[cfg(feature = "serde")]
pub use typetag;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod unary;

/**
This is a marker trait which defines trait bounds for all types `T` which can
be used as "quantities" in [`VarQuantity<T>`]. It does not provide any methods
and is auto-implemented for all `T` fulfilling the bounds, hence it is not
necessary to ever import this trait. It is only public to make compiler error
messages more helpful.
 */
pub trait IsQuantity:
    UnitFromType + TryFrom<DynQuantity<Complex<f64>>> + Clone + std::fmt::Debug
{
}

impl<T> IsQuantity for T where
    T: UnitFromType + TryFrom<DynQuantity<Complex<f64>>> + Clone + std::fmt::Debug
{
}

/**
Trait used to construct variable quantities whose value is a (pure) function of
other quantities.

Implementing this trait for a type marks it as being a variable quantity, whose
value can change under the influence of other quantities. For example, a
resistance can be a function of temperature:

```
use dyn_quantity::{DynQuantity, PredefUnit, Unit};
use var_quantity::QuantityFunction;

// The serde annotations are just here because the doctests of this crate use
// the serde feature - they are not needed if the serde feature is disabled.
#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct Resistance;

// Again, the macro annotation is just here because of the serde feature
#[typetag::serde]
impl QuantityFunction for Resistance {
    fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        let mut temperature = 0.0;
        let temperature_unit: Unit = PredefUnit::Temperature.into();
        for f in influencing_factors.iter() {
            if f.unit == temperature_unit {
                temperature = f.value;
                break;
            }
        }
        return DynQuantity::new(1.0 + temperature / 100.0, PredefUnit::ElectricResistance);
    }
}

// Influencing factors
let infl1 = &[DynQuantity::new(6.0, PredefUnit::ElectricCurrent)];
let infl2 = &[
    DynQuantity::new(6.0, PredefUnit::ElectricCurrent),
    DynQuantity::new(20.0, PredefUnit::Temperature),
];

let resistance = Resistance {};

assert_eq!(DynQuantity::new(1.0, PredefUnit::ElectricResistance), resistance.call(&[]));
assert_eq!(DynQuantity::new(1.0, PredefUnit::ElectricResistance), resistance.call(infl1));
assert_eq!(DynQuantity::new(1.2, PredefUnit::ElectricResistance), resistance.call(infl2));
```

An important constraint which unfortunately cannot be covered by the type
system is that the [`DynQuantity<f64>`] returned by [`QuantityFunction::call`]
must always have the same [`Unit`] field. See the [Features](#features) section
and the docstring of [`VarQuantity`] for details.

# Features

When the `serde` feature is enabled, any type implementing [`QuantityFunction`]
can be serialized / deserialized as a trait object using the
[typetag](https://docs.rs/typetag/latest/typetag/) crate. This has the following
implications:
- [`QuantityFunction::call`] cannot return a generic type (limitation of
typetag), which is why the dynamic [`DynQuantity`] type is used.
- When implementing [`QuantityFunction`] for a type, the `#[typetag::serde]`
annotation must be applied to the `impl` block (see example).

In turn, this feature enables serialization / deserialization of [`VarQuantity`]
without the need to specify the underlying function type in advance.
 */
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait QuantityFunction: dyn_clone::DynClone + Sync + Send + 'static {
    /**
    Returns a quantity as a function of `influencing_factors`. See the
    [`QuantityFunction`] trait docstring for examples.
    */
    fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64>;
}

/**
A thin wrapper around a `Box<dyn QuantityFunction>` trait object which provides
some type checks for usage in [`VarQuantity`].

This struct wraps a `Box<dyn QuantityFunction>` so it can be used in the
[`VarQuantity::Function`] enum variant. As explained in the [`QuantityFunction`]
docstring, the unit of the [`DynQuantity`] returned by [`QuantityFunction::call`]
must always be the same. Even though this can unfortunately not be represented
by the type system for reasons outlined in the trait docstring, this wrapper
provides some checks to reduce the likelihood of wrong units:
- When constructing the wrapper via [`FunctionWrapper::new`], it runs
[`QuantityFunction::call`] once with an empty slice and checks that the output unit
matches that of [`T::unit_from_type`](UnitFromType::unit_from_type). If that is
not the case, the construction fails and an error is returned.
- When calling the underlying function via [`FunctionWrapper::call`], it tries
to convert the [`DynQuantity<f64>`] delivered from [`QuantityFunction::call`]
into `T`. If that fails, the implementation of [`QuantityFunction`] violates
the requirement outlined in the trait documentation. This is a bug, hence the
function panics.

This struct has the same memory representation as [`Box<dyn QuantityFunction>`].
The underlying trait object can be accessed directly via [`FunctionWrapper::inner`].

# Features

This struct can be serialized / deserialized if the `serde` feature is enabled.
Since it is just a wrapper around a `Box<dyn QuantityFunction>` trait object,
it serializes directly to the representation of that object and deserializes
directly from it (it is["transparent"](https://serde.rs/container-attrs.html#transparent)).
 */
pub struct FunctionWrapper<T: IsQuantity> {
    function: Box<dyn QuantityFunction>,
    phantom: PhantomData<T>,
}

impl<T: IsQuantity> FunctionWrapper<T> {
    /**
    Creates a new instance of `Self` and performs a type safety check by running
    the [`QuantityFunction::call`] of `function` with an empty slice as
    `influencing_factors`. The unit of the resulting [`DynQuantity`] is then
    compared to that created by [`T::unit_from_type`](UnitFromType::unit_from_type).
    If they don't match, an error is returned. See the docstring of
    [`FunctionWrapper`] for more.

    # Examples

    ```
    use dyn_quantity::{DynQuantity, PredefUnit, Unit};
    use var_quantity::{QuantityFunction, FunctionWrapper};
    use uom::si::f64::{ElectricalResistance, ElectricCurrent};

    // The serde annotations are just here because the doctests of this crate use
    // the serde feature - they are not needed if the serde feature is disabled.
    #[derive(Clone, serde::Deserialize, serde::Serialize)]
    struct Resistance;

    // Again, the macro annotation is just here because of the serde feature
    #[typetag::serde]
    impl QuantityFunction for Resistance {
        fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
            return DynQuantity::new(1.0, PredefUnit::ElectricResistance);
        }
    }

    let resistance = Resistance {};

    // The Resistance struct always returns an electric resistance. Hence the
    // type check fails for other types
    assert!(FunctionWrapper::<ElectricalResistance>::new(Box::new(resistance.clone())).is_ok());
    assert!(FunctionWrapper::<f64>::new(Box::new(resistance.clone())).is_err());
    assert!(FunctionWrapper::<ElectricCurrent>::new(Box::new(resistance.clone())).is_err());
    ```
     */
    pub fn new(function: Box<dyn QuantityFunction>) -> Result<Self, UnitsNotEqual> {
        // Call the function w/o any arguments and make sure the returned
        // DynQuantity<f64> is convertible to T
        let actual = function.call(&[]).unit;
        let expected = T::unit_from_type();
        if actual != expected {
            return Err(UnitsNotEqual(expected, actual));
        }
        return Ok(Self {
            function,
            phantom: PhantomData,
        });
    }

    /**
    Forwards the input to the [`QuantityFunction::call`] method of the wrapped
    trait object and asserts that the returned value can be converted to `T`.
    If that is not the case, the constraint outlined in the docstring of
    [`FunctionWrapper`] is not fulfilled and the code is invalid, therefore
    the function panics.

    # Examples

    This is a valid implementation of [`IsQuantity`]: [`Unit`] is always the
    same regardless of input.
    ```
    use dyn_quantity::{DynQuantity, PredefUnit, Unit};
    use var_quantity::{QuantityFunction, FunctionWrapper};
    use uom::si::electrical_resistance::ohm;
    use uom::si::f64::{ElectricalResistance};

    // The serde annotations are just here because the doctests of this crate use
    // the serde feature - they are not needed if the serde feature is disabled.
    #[derive(Clone, serde::Deserialize, serde::Serialize)]
    struct Resistance;

    // Again, the macro annotation is just here because of the serde feature
    #[typetag::serde]
    impl QuantityFunction for Resistance {
        fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
            return DynQuantity::new(1.0, PredefUnit::ElectricResistance);
        }
    }

    let wrapped_resistance = FunctionWrapper::<ElectricalResistance>::new(Box::new(Resistance {})).expect("units match");
    assert_eq!(ElectricalResistance::new::<ohm>(1.0), wrapped_resistance.call(&[1.0.into()]));
    ```

    This is an invalid (and nonsensical) implementation of [`QuantityFunction`]
    where the output unit changes with the number of arguments:
    ```should_panic
    use dyn_quantity::{DynQuantity, PredefUnit, Unit};
    use var_quantity::{QuantityFunction, FunctionWrapper};
    use uom::si::f64::{ElectricalResistance};

    // The serde annotations are just here because the doctests of this crate use
    // the serde feature - they are not needed if the serde feature is disabled.
    #[derive(Clone, serde::Deserialize, serde::Serialize)]
    struct Resistance;

    // Again, the macro annotation is just here because of the serde feature
    #[typetag::serde]
    impl QuantityFunction for Resistance {
        fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
            if influencing_factors.len() == 0 {
                return DynQuantity::new(1.0, PredefUnit::ElectricResistance);
            } else {
                return DynQuantity::new(1.0, PredefUnit::None);
            }
        }
    }

    // Construction succeeds since the test call is done with an empty slice
    let wrapped_resistance = FunctionWrapper::<ElectricalResistance>::new(Box::new(Resistance {})).expect("units match");

    // ... but calling with a quantity results in a panic
    let _ = wrapped_resistance.call(&[DynQuantity::new(1.0, PredefUnit::None)]);
    ```
     */
    pub fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> T {
        match T::try_from(self.function.call(influencing_factors).into()) {
            Ok(val) => val,
            Err(_) => {
                panic!(
                    "conversion from DynQuantity<f64> to T failed for input {:?}.\n
                    This means that the QuantityFunction trait object returns
                    different DynQuantity<f64> depending on the input, which
                    is a bug in the implementation of the trait object.",
                    influencing_factors
                )
            }
        }
    }

    /**
    Returns the underlying [`QuantityFunction`] trait object.
     */
    pub fn inner(&self) -> &dyn QuantityFunction {
        return &*self.function;
    }
}

impl<T: IsQuantity> std::fmt::Debug for FunctionWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FunctionWrapper").finish()
    }
}

impl<T: IsQuantity> Clone for FunctionWrapper<T> {
    fn clone(&self) -> Self {
        return Self {
            function: dyn_clone::clone_box(&*self.function),
            phantom: PhantomData,
        };
    }
}

/**
A quantity whose value can either be constant or a function of one or more other
quantities.

The value of (physical) quantities can depend on the values of other quantities.
This is often the case for quantities representing physical properties such as
e.g. the electric resistance of a conductor. This enum serves as a general
container for such quantities with the variant [`VarQuantity::Constant`] being
an optimization for the important case of a constant quantitity and with the
variant [`VarQuantity::Function`] covering all other cases via a
[`QuantityFunction`] trait object (wrapped in [`FunctionWrapper`]). Due to the
generic design, it can also be used for dimensionless quantities which can be
represented by a simple [`f64`].

The value of the underlying quantity can be read out via the [`VarQuantity::get`]
method. It takes a slice of [`DynQuantity`] representing influencing factors,
for example the temperature in case of a resistance. If the enum variant is
constant, the value field is simply cloned, otherwise the [`QuantityFunction::call`]
function is called. This returns a [`DynQuantity<f64>`], which must be convertable
via [`TryFrom`] to `T` (enforced by trait bound). This dynamic approach is
chosen to make this enum serializable / deserializable (see section
[Features](#features)).

Even though the conversion from [`DynQuantity<f64>`] to `T` is fallible from the
perspective of the type system, in actual implementations it must be infallible
(i.e. the conversion must always succeed). This is done so `T` can be a
statically typed physical quantity (e.g. from the [uom](https://crates.io/crates/uom)
library), for which [`From<DynQuantity<f64>>`] can obviously not be implemented.
The conversion is checked once when constructing a [`FunctionWrapper`] from a
[`QuantityFunction`] trait object by calling [`QuantityFunction::call`] with
`influencing_factors = &[]`, but of course it is impossible to test all
potential values for `influencing_factors`.

It is therefore up to the provider of the trait object to make sure that the
[`DynQuantity<f64>`] returned by [`QuantityFunction::call`] always has the same
[`Unit`]. If this is not the case, the trait object has a bug and the program
has entered an invalid state, resulting in a [`panic!`].

# Examples

## f64 and statically typed physical quantities

This example shows how [`VarQuantity`] integrates with both [`f64`] and
[uom](https://crates.io/crates/uom) [`Quantity`](https://docs.rs/uom/latest/uom/si/struct.Quantity.html).
```
use dyn_quantity::{DynQuantity, PredefUnit, Unit};
use uom::si::electrical_resistance::ohm;
use uom::si::f64::ElectricalResistance;
use var_quantity::{FunctionWrapper, QuantityFunction, VarQuantity};

// =============================================================================
// Constant quantity with f64
let qt_const = VarQuantity::<f64>::Constant(2.0);

// Influencing factors
let infl1 = &[DynQuantity::new(6.0, PredefUnit::ElectricCurrent)];
let infl2 = &[
    DynQuantity::new(6.0, PredefUnit::ElectricCurrent),
    DynQuantity::new(20.0, PredefUnit::Temperature),
];

// Since this is a constant quantity, it returns always 2 regardless of the input.
assert_eq!(2.0, qt_const.get(&[]));
assert_eq!(2.0, qt_const.get(infl1));
assert_eq!(2.0, qt_const.get(infl2));

// =============================================================================
// Variable quantity

// A variable resistance: The resistance is 1 + temperature / 100.
// For the test, the serde feature is enabled, hence it is necessary to
// implement serialization and deserialization as well as #[typetag::serde].
// This is not needed if the feature is not enabled.
#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct ResistanceFunction;

#[typetag::serde]
impl QuantityFunction for ResistanceFunction {
    fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        let mut temperature = 0.0;
        let temperature_unit: Unit = PredefUnit::Temperature.into();
        for f in influencing_factors.iter() {
            if f.unit == temperature_unit {
                temperature = f.value;
                break;
            }
        }
        return DynQuantity::new(1.0 + temperature / 100.0, PredefUnit::ElectricResistance);
    }
}

let wrapper = FunctionWrapper::new(Box::new(ResistanceFunction {})).expect("type check successfull");
let qt_var = VarQuantity::<ElectricalResistance>::Function(wrapper);

// Input infl2 contains a temperature and therefore influences the resistance.
assert_eq!(ElectricalResistance::new::<ohm>(1.0), qt_var.get(&[]));
assert_eq!(ElectricalResistance::new::<ohm>(1.0), qt_var.get(infl1));
assert_eq!(ElectricalResistance::new::<ohm>(1.2), qt_var.get(infl2));
```

## Unit mismatch

This example shows a violation of the assumption that the [`DynQuantity`] returned
by the [`QuantityFunction`] trait object is convertible to `T`.
```
use dyn_quantity::{DynQuantity, PredefUnit};
use uom::si::electrical_conductance::siemens;
use uom::si::f64::{ElectricalResistance, ElectricalConductance};
use var_quantity::{FunctionWrapper, QuantityFunction, VarQuantity};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct ResistanceFunction;

#[typetag::serde]
impl QuantityFunction for ResistanceFunction {
    fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        return DynQuantity::new(1.0, PredefUnit::ElectricResistance);
    }
}

// Mismatch in type definition - catched during construction of FunctionWrapper
let wrapper = FunctionWrapper::<ElectricalConductance>::new(Box::new(ResistanceFunction {}));
assert!(wrapper.is_err());
```

# Features

If the `serde` feature is activated, this enum can be serialized and
deserialized (as untagged enum). The [`QuantityFunction`] trait object is
serialized / deserialized using [typetag](https://docs.rs/typetag/latest/typetag/).
This is also the reason why [`QuantityFunction::call`] returns a
[`DynQuantity<f64>`] instead of a generic type.
 */
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum VarQuantity<T: IsQuantity> {
    /**
    Optimization for the common case of a constant quantity. This avoids going
    through dynamic dispatch when accessing the value.
     */
    Constant(T),
    /**
    Catch-all variant for any non-constant behaviour. Arbitrary behaviour
    can be realized with the contained [`QuantityFunction`] trait object, as
    long as the unit constraint outlined in the [`VarQuantity`] docstring is
    upheld.
     */
    Function(FunctionWrapper<T>),
}

impl<T: IsQuantity> VarQuantity<T> {
    /**
    Matches against `self` and either returns the contained value (variant
    [`VarQuantity::Constant`]) or executes the call method of the contained
    [`FunctionWrapper`] (variant [`VarQuantity::Function`]).
    */
    pub fn get(&self, influencing_factors: &[DynQuantity<f64>]) -> T {
        match self {
            Self::Constant(val) => val.clone(),
            Self::Function(fun) => fun.call(influencing_factors),
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use serde::de::DeserializeOwned;

    use super::*;

    impl<T: IsQuantity> Serialize for FunctionWrapper<T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.function.serialize(serializer)
        }
    }

    impl<'de, T: IsQuantity> serde::Deserialize<'de> for FunctionWrapper<T> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let v = <Box<dyn QuantityFunction>>::deserialize(deserializer)?;
            FunctionWrapper::new(v).map_err(serde::de::Error::custom)
        }
    }

    impl<'de, T> serde::Deserialize<'de> for VarQuantity<T>
    where
        T: DeserializeOwned + IsQuantity,
        <T as TryFrom<DynQuantity<Complex<f64>>>>::Error: std::fmt::Display,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use std::str::FromStr;

            #[derive(deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError)]
            enum NumberOrString<T> {
                Number(T),
                #[cfg(feature = "from_str")]
                String(String),
            }

            let content: serde_value::Value = serde::Deserialize::deserialize(deserializer)?;

            // Try to deserialize as a quantity. If that fails, try to deserialize as a function trait object
            match NumberOrString::<T>::deserialize(serde_value::ValueDeserializer::<D::Error>::new(
                content.clone(),
            )) {
                Ok(number_or_string) => match number_or_string {
                    NumberOrString::Number(q) => return Ok(VarQuantity::Constant(q)),
                    NumberOrString::String(s) => {
                        let dq = DynQuantity::<Complex<f64>>::from_str(&s)
                            .map_err(serde::de::Error::custom)?;
                        let q = T::try_from(dq).map_err(serde::de::Error::custom)?;
                        return Ok(VarQuantity::Constant(q));
                    }
                },
                Err(_) => {
                    let wrapper =
                        FunctionWrapper::deserialize(
                            serde_value::ValueDeserializer::<D::Error>::new(content.clone()),
                        )?;
                    return Ok(VarQuantity::Function(wrapper));
                }
            }
        }
    }
}

/**
A wrapper around a type implementing [`QuantityFunction`] trait object which
clamps the output of [`QuantityFunction::call`] using the provided upper and
lower limits.

If the `serde` feature is not activated, it implements [`QuantityFunction`]
in a generic manner and can therefore be used in a [`FunctionWrapper`]. If
`serde` is activated, it is unfortately not possible to provide a generic
implementation due to the macro `#[typetag::serde]` not being able to deal with
generics. As a workaround, it is possible to provide a simple custom
implementation for each concrete type in your own crate:

```ignore
#[cfg_attr(feature = "serde", typetag::serde)]
impl QuantityFunction for ClampedQuantity<YourTypeHere> {
    fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        return self.call_clamped(influencing_factors);
    }
}
```

This approach is used for all the implementors of [`QuantityFunction`] provided
with this crate.
 */
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ClampedQuantity<T: QuantityFunction> {
    upper_limit: f64,
    lower_limit: f64,
    function: T,
}

impl<T: QuantityFunction> ClampedQuantity<T> {
    /**
    Checks if `upper_limit >= lower_limit` and returns a new instance of
    [`ClampedQuantity`] if true.
    */
    pub fn new(upper_limit: f64, lower_limit: f64, function: T) -> Result<Self, &'static str> {
        if upper_limit < lower_limit {
            return Err("upper limit must not be smaller than the lower limit");
        }
        return Ok(Self {
            upper_limit,
            lower_limit,
            function,
        });
    }

    /**
    Returns the underlying [`QuantityFunction`].
     */
    pub fn inner(&self) -> &T {
        return &self.function;
    }

    /**
    Returns the underlying [`QuantityFunction`] as a trait object.
     */
    pub fn inner_dyn(&self) -> &dyn QuantityFunction {
        return &self.function;
    }

    /// Returns the upper limit.
    pub fn upper_limit(&self) -> f64 {
        return self.upper_limit;
    }

    /// Returns the lower limit.
    pub fn lower_limit(&self) -> f64 {
        return self.lower_limit;
    }

    /**
    Clamps the output value of `T::call` using the provided upper and lower
    limits. This function is mainly here to simplify custom [`QuantityFunction`]
    implementations, see the [`ClampedQuantity`] docstring.
     */
    pub fn call_clamped(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        let mut dyn_quantity = self.function.call(influencing_factors);
        dyn_quantity.value = dyn_quantity.value.clamp(self.lower_limit, self.upper_limit);
        return dyn_quantity;
    }
}

#[cfg(not(feature = "serde"))]
impl<T: QuantityFunction> QuantityFunction for ClampedQuantity<T> {
    fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        return self.call_clamped(influencing_factors);
    }
}

/**
A helper function which filters the `influencing_factors` for a quantity with
the type `match_for`. If a matching quantity is found, it is used as argument
for `F` and the result is returned. Otherwise, the result of `G()` is returned.

The main purpose of this function is to simplify writing unary functions. For
example, the [`QuantityFunction::call`] implementation of a linear function
can look like this:

```
use dyn_quantity::{DynQuantity, Unit};
use var_quantity::{filter_unary_function, QuantityFunction};

// The serde annotations are just here because the doctests of this crate use
// the serde feature - they are not needed if the serde feature is disabled.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Linear {
    slope: f64,
    base_value: f64,
}

// Again, the macro annotation is just here because of the serde feature
#[cfg_attr(feature = "serde", typetag::serde)]
impl QuantityFunction for Linear {
    fn call(&self, influencing_factors: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        return filter_unary_function(
            influencing_factors,
            Unit::default(),
            |input| {
                DynQuantity::new(
                    self.base_value + self.slope * input.value,
                    Unit::default(),
                )
            },
            || DynQuantity::new(
                    self.base_value,
                    Unit::default(),
                ),
        );
    }
}
```
 */
pub fn filter_unary_function<F, G>(
    influencing_factors: &[DynQuantity<f64>],
    match_for: Unit,
    with_matched: F,
    no_match: G,
) -> DynQuantity<f64>
where
    F: FnOnce(DynQuantity<f64>) -> DynQuantity<f64>,
    G: FnOnce() -> DynQuantity<f64>,
{
    for iq in influencing_factors {
        if iq.unit == match_for {
            return with_matched(iq.clone());
        }
    }
    no_match()
}
