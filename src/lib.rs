/*!
[`VarQuantity`]: crate::VarQuantity
[`VarQuantity::eval`]: crate::VarQuantity::eval
[`VarQuantity::new`]: crate::VarQuantity::new
[`QuantityFunction`]: crate::QuantityFunction
[`QuantityFunction::eval`]: crate::QuantityFunction::eval
[`filter_unary_function`]: crate::filter_unary_function
[`FirstOrderTaylor`]: crate::unary::FirstOrderTaylor
[`Polynomial`]: crate::unary::Polynomial
[`unary`]: crate::unary

This crate provides a composable interface for defining and evaluating variable
physical quantities whose values depend on other quantities.
 */
#![doc = include_str!("../docs/main.md")]
#![deny(missing_docs)]

use std::marker::PhantomData;
use std::ops::{Add, Div, Mul, Sub};

// Prevent rustdoc from documenting the dependency again
#[doc(hidden)]
pub use dyn_quantity::*;
use uom::typenum::{Integer, Prod};

use num::Complex;

#[doc(hidden)]
#[cfg(feature = "serde")]
pub use typetag;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use dyn_quantity;
pub mod unary;

/**
This is a marker trait which defines trait bounds for all types `T` which can
be used as "quantities" in [`VarQuantity<T>`]. It does not provide any methods
and is auto-implemented for all `T` fulfilling the bounds, hence it is not
necessary to ever import this trait. It is only public to make compiler error
messages more helpful.
 */
#[cfg(not(feature = "serde"))]
pub trait QuantityF64:
    UnitFromType
    + TryFrom<DynQuantity<Complex<f64>>>
    + TryFrom<DynQuantity<f64>>
    + Clone
    + std::fmt::Debug
    + Into<DynQuantity<Complex<f64>>>
    + Into<DynQuantity<f64>>
    + PartialEq
    + Default
{
}

/**
This is a marker trait which defines trait bounds for all types `T` which can
be used as "quantities" in [`VarQuantity<T>`]. It does not provide any methods
and is auto-implemented for all `T` fulfilling the bounds, hence it is not
necessary to ever import this trait. It is only public to make compiler error
messages more helpful.
 */
#[cfg(feature = "serde")]
pub trait QuantityF64:
    UnitFromType
    + TryFrom<DynQuantity<Complex<f64>>>
    + TryFrom<DynQuantity<f64>>
    + Clone
    + std::fmt::Debug
    + Into<DynQuantity<Complex<f64>>>
    + Into<DynQuantity<f64>>
    + Serialize
    + PartialEq
    + Default
{
}

#[cfg(feature = "serde")]
impl<T> QuantityF64 for T where
    T: UnitFromType
        + TryFrom<DynQuantity<Complex<f64>>>
        + TryFrom<DynQuantity<f64>>
        + Clone
        + std::fmt::Debug
        + Into<DynQuantity<Complex<f64>>>
        + Into<DynQuantity<f64>>
        + serde::Serialize
        + PartialEq
        + Default
{
}

#[cfg(not(feature = "serde"))]
impl<T> QuantityF64 for T where
    T: UnitFromType
        + TryFrom<DynQuantity<Complex<f64>>>
        + TryFrom<DynQuantity<f64>>
        + Clone
        + std::fmt::Debug
        + Into<DynQuantity<Complex<f64>>>
        + Into<DynQuantity<f64>>
        + PartialEq
        + Default
{
}

/**
Trait for defining models of physical quantities whose values depend on other
quantities.

Implementing this trait allows a type to represent a variable quantity that can
be composed with other quantities using [`VarQuantity`]. For example, electrical
resistance can depend on temperature:

```
use var_quantity::{DynQuantity, PredefUnit, Unit, QuantityFunction};

// The serde annotations are just here because the doctests of this crate use
// the serde feature - they are not needed if the serde feature is disabled.
#[derive(Clone, serde::Deserialize, serde::Serialize, PartialEq)]
struct Resistance;

// Again, the macro annotation is just here because of the serde feature
#[typetag::serde]
impl QuantityFunction for Resistance {
    fn eval(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        let mut temperature = 0.0;
        let mut temperature = 0.0;
        let temperature_unit: Unit = PredefUnit::Temperature.into();

        for condition in conditions {
            if condition.unit == temperature_unit {
                temperature = condition.value;
                break;
            }
        }
        DynQuantity::new(1.0 + temperature / 100.0, PredefUnit::ElectricalResistance)
    }

    fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
        (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
    }
}

// Environmental conditions
let conditions_without_temperature = &[
    DynQuantity::new(6.0, PredefUnit::ElectricCurrent),
];

let conditions_with_temperature = &[
    DynQuantity::new(6.0, PredefUnit::ElectricCurrent),
    DynQuantity::new(20.0, PredefUnit::Temperature),
];

let resistance = Resistance {};

assert_eq!(DynQuantity::new(1.0, PredefUnit::ElectricalResistance), resistance.eval(&[]));
assert_eq!(DynQuantity::new(1.0, PredefUnit::ElectricalResistance), resistance.eval(conditions_without_temperature));
assert_eq!(DynQuantity::new(1.2, PredefUnit::ElectricalResistance), resistance.eval(conditions_with_temperature));
```

An important constraint which unfortunately cannot be covered by the type
system is that the [`DynQuantity<f64>`] returned by [`QuantityFunction::eval`]
must always have the same [`Unit`] field. See the [Features](#features) section
and the docstring of [`VarQuantity`] for details.

# Serialization and deserialization

When the `serde` feature is enabled, [`QuantityFunction`] implementations
can be serialized and deserialized as trait objects using the
[`typetag`](https://docs.rs/typetag/latest/typetag/) crate.

To enable this functionality, an implementation must apply the
`#[typetag::serde]` attribute to its `impl` block (see the example above)
and the implementing type must implement Serde's serialization and
deserialization traits.

Because [`QuantityFunction`] is used as a trait object, its methods must
be object-safe. In particular, [`QuantityFunction::eval`] cannot have a
generic return type. Instead, it returns [`DynQuantity`], which provides
a dynamically represented quantity.

This integration allows [`VarQuantity`] expressions to be serialized and
deserialized without requiring the concrete types of their underlying
functions to be known in advance.
 */
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait QuantityFunction: dyn_clone::DynClone + Sync + Send + std::any::Any {
    /**
    Evaluates the function using the supplied environmental `conditions`.

    The `conditions` slice contains the quantities available to the caller.
    The implementation is responsible for extracting the quantities relevant
    to its model and determining how to handle missing conditions.

    When [`VarQuantity::<T>::new`] is called with an implementation of this
    trait, this function is evaluated with an empty conditions slice. The
    returned quantity is checked for compatibility with `T`. If it cannot be
    converted, construction fails.

    The returned [`DynQuantity`] must always have the same [`Unit`],
    regardless of the supplied conditions. This requirement cannot be enforced
    by the type system. If an implementation violates it, [`VarQuantity::eval`]
    returns the default value of `T` when the returned quantity cannot be
    converted. [`VarQuantity::eval_checked`] returns an error in this case, see
    its docstring for an example.

    See the [`QuantityFunction`] trait documentation for an example.
    */
    fn eval(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64>;

    /**
    Determines whether `self` and `other` are interchangeable for evaluation.

    During [`VarQuantity::eval`], equivalent function instances are deduplicated,
    so [`QuantityFunction::eval`] is called at most once for equivalent
    instances. This method determines whether two instances are considered
    equivalent for this purpose. This method is also used by the [`PartialEq`]
    implementations of [`RpnToken::OperandFn`] and, consequently,
    [`VarQuantity`].

    Implementations must return `true` only if evaluating either instance would
    produce equivalent results under the same conditions. In particular,
    implementations whose evaluation results depend on their internal state
    must not consider instances with potentially different results equivalent.

    Since [`QuantityFunction::eval`] takes a shared reference, state-dependent
    evaluation requires interior mutability, for example through a
    [`Mutex`](std::sync::Mutex) .[`Cell`](std::cell::Cell) and
    [`RefCell`](std::cell::RefCell) cannot be as part of `self` because they do
    not implement [`Sync`], which is a supertrait of [`QuantityFunction`].

    Returning `false` prevents the instances from being deduplicated.

    For a [`Sized`] type which implements [`PartialEq`], this method can be
    implemented as:

    ```ignore
    fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
        (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
    }
    ```
    If `Self` cannot be compared, this method should simply return `false`.
    */
    fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool;
}

dyn_clone::clone_trait_object!(QuantityFunction);

/// A token in the Reverse Polish Notation (RPN) representation of a
/// [`VarQuantity<T>`].
///
/// See [`VarQuantity::from_tokens`] for an usage example.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RpnToken {
    /// A boxed [`QuantityFunction`] which is an operand of the composed function.
    OperandFn(Box<dyn QuantityFunction>),
    /// References a quantity function by its index in the function cache.
    ///
    /// The index corresponds to the zero-based position of the function among
    /// all [`RpnToken::OperandFn`] tokens in the expression. Unlike a token
    /// index, it can therefore be used directly to access the corresponding
    /// entry in the evaluator's function cache.
    ///
    /// When building a token sequence manually, it is generally not necessary
    /// to use this token, as [`VarQuantity::from_tokens`] performs
    /// deduplication of function operands automatically (see the documentation
    /// of [`VarQuantity`] for an example). If a [`RpnToken::OperandFnRef`] has
    /// no corresponding [`RpnToken::OperandFn`] in the token sequence,
    /// evaluation fails with [`EvalError::InvalidFunctionReference`]. This can
    /// only happen if the token has been created explicitly.
    ///
    /// As an example, take the following token sequence:
    ///
    /// `f0, f1, -, ref2, +`
    ///
    /// with `ref2` being a `RpnTokenOperand(2)`. Since there are only two
    /// functions, attempting to build a [`VarQuantity`] from this will return
    /// the aforementioned [`EvalError::InvalidFunctionReference`].
    OperandFnRef(usize),
    /// Represents a constant quantity that is used directly during evaluation.
    OperandConst(DynQuantity<f64>),
    /// Clamps the preceding operand to the inclusive range [`min`, `max`].
    Clamp {
        /// Lower bound of the range.
        min: f64,
        /// Upper bound of the range.
        max: f64,
    },
    /// Raises the preceding operand to the integer power.
    Power(i8),
    /// Takes the absolute value of the preceding operand
    Abs,
    /// Adds the two preceding operands.
    Add,
    /// Subtracts the preceding operand from the second-to-last operand.
    Sub,
    /// Multiplies the two preceding operands.
    Mul,
    /// Divides the second-to-last operand by the preceding operand.
    Div,
}

impl PartialEq for RpnToken {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::OperandFn(l0), Self::OperandFn(r0)) => l0.dyn_eq(&**r0),
            (Self::OperandFnRef(l0), Self::OperandFnRef(r0)) => l0 == r0,
            (Self::OperandConst(l0), Self::OperandConst(r0)) => l0 == r0,
            (
                Self::Clamp {
                    min: lmin,
                    max: lmax,
                },
                Self::Clamp {
                    min: rmin,
                    max: rmax,
                },
            ) => lmin == rmin && lmax == rmax,
            (Self::Power(l0), Self::Power(r0)) => l0 == r0,
            (Self::Abs, Self::Abs) => true,
            (Self::Add, Self::Add) => true,
            (Self::Sub, Self::Sub) => true,
            (Self::Mul, Self::Mul) => true,
            (Self::Div, Self::Div) => true,
            _ => false,
        }
    }
}

impl std::fmt::Debug for RpnToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OperandFn(_) => f.debug_tuple("OperandFn").finish(),
            Self::OperandFnRef(arg0) => f.debug_tuple("OperandFnRef").field(arg0).finish(),
            Self::OperandConst(arg0) => f.debug_tuple("OperandConst").field(arg0).finish(),
            Self::Clamp { min, max } => f
                .debug_struct("Clamp")
                .field("min", min)
                .field("max", max)
                .finish(),
            Self::Power(arg0) => f.debug_tuple("Power").field(arg0).finish(),
            Self::Abs => write!(f, "Abs"),
            Self::Add => write!(f, "Add"),
            Self::Sub => write!(f, "Sub"),
            Self::Mul => write!(f, "Mul"),
            Self::Div => write!(f, "Div"),
        }
    }
}

impl From<Box<dyn QuantityFunction>> for RpnToken {
    fn from(value: Box<dyn QuantityFunction>) -> Self {
        Self::OperandFn(value)
    }
}

/**
Errors that can occur when evaluating a [`VarQuantity`] expression.

These errors indicate an invalid token sequence or an incompatibility between
quantities encountered during evaluation. It is created by
[`VarQuantity::from_tokens`] and
[`VarQuantity::eval_checked`] if the token sequence cannot be evaluated.
*/
#[derive(Debug)]
pub enum EvalError {
    /// The expression contains no tokens.
    EmptyExpression,
    /// An operator does not have enough operands preceding it.
    MissingOperand {
        /// Index of the operator in the token sequence.
        token_index: usize,
        /// Name of the operator.
        operator: &'static str,
        /// Number of operands required by the operator.
        required: usize,
        /// Number of operands available when the operator is encountered.
        available: usize,
    },
    /// A function reference token [`RpnToken::OperandFnRef`] does not refer to
    /// a function in the expression.
    InvalidFunctionReference {
        /// Index of the reference token in the token sequence.
        token_index: usize,
        /// Index of the function in order of function appearances. See
        /// [`RpnToken::OperandFnRef`].
        function_index: usize,
    },
    /// The expression does not leave exactly one value on the evaluation stack.
    InvalidStackDepth {
        /// Number of values remaining on the evaluation stack.
        depth: usize,
    },
    /// An operation requires a stack value that is not available.
    MissingStackValue {
        /// Index of the token being evaluated.
        token_index: usize,
        /// Index of the missing value on the evaluation stack.
        stack_index: usize,
    },
    /// Two quantities have incompatible units.
    UnitsNotEqual {
        /// Index of the token at which the incompatibility was detected.
        token_index: usize,
        /// Details of the unit incompatibility.
        source: UnitsNotEqual,
    },
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyExpression => {
                write!(f, "cannot evaluate an empty expression")
            }
            Self::MissingOperand {
                token_index,
                operator,
                required,
                available,
            } => {
                write!(
                    f,
                    "operator '{operator}' at token index {token_index} requires \
                     {required} operands, but only {available} available"
                )
            }
            Self::InvalidFunctionReference {
                token_index,
                function_index,
            } => {
                write!(
                    f,
                    "function reference at token index {token_index} refers to \
                     nonexistent function at index {function_index}"
                )
            }
            Self::InvalidStackDepth { depth } => {
                write!(
                    f,
                    "expression evaluation finished with an invalid stack depth \
                     of {depth}; expected exactly one value"
                )
            }
            Self::MissingStackValue {
                token_index,
                stack_index,
            } => {
                write!(
                    f,
                    "token at index {token_index} references missing stack value \
                     at index {stack_index}"
                )
            }
            Self::UnitsNotEqual {
                token_index,
                source,
            } => {
                write!(f, "unit mismatch at token index {token_index}: {source}")
            }
        }
    }
}

impl std::error::Error for EvalError {}

/// Information extracted from the token stream used in [`VarQuantity::eval_checked`].
struct TokenAnalysis {
    max_stack_depth: usize,
    function_count: usize,
}

/// Analyses the token stream to determine the size of the array used in [`VarQuantity::eval_checked`].
///
/// If the analysis shows the token stream is invalid, this information is also bubbled up.
fn analyse_tokens(tokens: &[RpnToken]) -> Result<TokenAnalysis, EvalError> {
    let mut depth = 0;
    let mut max_stack_depth = 0;
    let mut function_count = 0;

    for (token_index, token) in tokens.iter().enumerate() {
        match token {
            RpnToken::OperandFn(_) => {
                depth += 1;
                function_count += 1;
            }
            RpnToken::OperandFnRef(index) => {
                if *index >= function_count {
                    return Err(EvalError::InvalidFunctionReference {
                        token_index,
                        function_index: *index,
                    });
                }

                depth += 1;
            }
            RpnToken::OperandConst(_) => {
                depth += 1;
            }
            RpnToken::Clamp { min: _, max: _ } | RpnToken::Power(_) | RpnToken::Abs => {
                if depth < 1 {
                    return Err(EvalError::MissingOperand {
                        token_index,
                        operator: "unary",
                        required: 1,
                        available: depth,
                    });
                }
            }
            RpnToken::Add | RpnToken::Sub | RpnToken::Mul | RpnToken::Div => {
                if depth < 2 {
                    return Err(EvalError::MissingOperand {
                        token_index,
                        operator: "binary",
                        required: 2,
                        available: depth,
                    });
                }

                depth -= 1;
            }
        }

        max_stack_depth = max_stack_depth.max(depth);
    }

    if depth != 1 {
        return Err(EvalError::InvalidStackDepth { depth });
    }

    Ok(TokenAnalysis {
        max_stack_depth,
        function_count,
    })
}

/// Replaces duplicate function operands with references to their first occurrence.
///
/// Function operands are compared using [`RpnToken`]'s equality implementation.
/// If an equal function operand already exists in `tokens`, `token` is replaced
/// with an [`RpnToken::OperandFnRef`] pointing to that operand.
///
/// Consequently, function references always point to an earlier token in the
/// expression. This invariant allows the evaluator to resolve function
/// references using the cache index assigned when the original operand was
/// encountered, without requiring an additional lookup table.
fn deduplicate_fn_tokens(tokens: &[RpnToken], token: &mut RpnToken) {
    if let RpnToken::OperandFn(_) = token {
        let mut function_count = 0;

        for t in tokens.iter() {
            if t == token {
                *token = RpnToken::OperandFnRef(function_count);
                return;
            }

            if matches!(t, RpnToken::OperandFn(_)) {
                function_count += 1;
            }
        }
    }
}

/**
A physical quantity that can either be constant or depend on other quantities
through a composition of [`QuantityFunction`]s.

This is the central type of this crate, enabling constants and
[`QuantityFunction`] implementations to be composed into new functions
using arithmetic operators.

A [`VarQuantity`] can be created directly from a constant of type `T` using
[`VarQuantity::from`], from a [`QuantityFunction`] implementor using
[`VarQuantity::new`], or from a sequence of [`RpnToken`]s using
[`VarQuantity::from_tokens`]. Arithmetic operations on [`VarQuantity`]
instances produce new [`VarQuantity`] expressions:

```
use approxim::assert_abs_diff_eq;

use var_quantity::unary::FirstOrderTaylor;
use var_quantity::uom::si::f64::*;
use var_quantity::uom::si::{
    electric_current::ampere, electric_potential::volt, electrical_resistance::ohm,
    temperature_interval::kelvin, thermodynamic_temperature::degree_celsius,
};
use var_quantity::*;

// Resistor whose temperature dependency can be expressed as a first-order Taylor approximation:
// r = r0 * (1 + k * (t - t0))
let r0 = ElectricalResistance::new::<ohm>(10.0);
let k = 0.005 / TemperatureInterval::new::<kelvin>(1.0);
let t0 = ThermodynamicTemperature::new::<degree_celsius>(20.0);
let model = FirstOrderTaylor::new(r0.into(), k.into(), t0.into()).expect("k * t0 is dimensionless");

let r: VarQuantity<ElectricalResistance> = VarQuantity::new(model).expect("model output must have units of resistance");
let u: VarQuantity<ElectricPotential> = VarQuantity::from(ElectricPotential::new::<volt>(30.0));
let i: VarQuantity<ElectricCurrent> = u / r;

// Evaluation at different temperatures
assert_abs_diff_eq!(i.eval(&[ThermodynamicTemperature::new::<degree_celsius>(20.0).into()]).get::<ampere>(), 3.0, epsilon = 1e-3);
assert_abs_diff_eq!(i.eval(&[ThermodynamicTemperature::new::<degree_celsius>(120.0).into()]).get::<ampere>(), 2.0, epsilon = 1e-3);

// Evaluation without providing a temperature. The behaviour depends on the
// underlying model; this model assumes room temperature (20 °C).
assert_abs_diff_eq!(i.eval(&[]).get::<ampere>(), 3.0, epsilon = 1e-3);
```

As shown in the example, the [`eval`](VarQuantity::eval) method evaluates the
[`VarQuantity`] for the supplied environmental conditions and returns the
resulting quantity. During evaluation, each unique [`QuantityFunction`] is
evaluated at most once, even if it occurs multiple times in the expression.
Function instances are considered identical according to
[`QuantityFunction::dyn_eq`].

As an example, take the following token sequence as an input to
[`VarQuantity::from_tokens`] or to an deserializer:

`f0, f1, +, f2, -, f0, +, f2, +`

Passing over the sequence reveals that `f0` and `f2` occur twice. The second
occurrence of each is replaced with [`RpnToken::OperandFnRef`] tokens referring
to the first occurrence (`RpnToken::OperandFnRef(0)` and
`RpnToken::OperandFnRef(2)` respectively). The resulting token sequence is:

`f0, f1, +, f2, -, ref0, +, ref2, +`

The supplied conditions do not need to cover every dependency of the underlying
models. Each [`QuantityFunction`] implementation determines which conditions it
uses and how missing conditions are handled.

# Internal representation

Internally, a VarQuantity expression is represented by a
[`Vec<RpnToken>`] containing [`RpnToken`]s in
[Reverse Polish Notation (RPN)](https://en.wikipedia.org/wiki/Reverse_Polish_notation).
The token sequence can be inspected using [`VarQuantity::tokens`], but cannot be
modified directly. To extend an expression, use the arithmetic operators or
composition methods such as [`VarQuantity::clamp`] and [`VarQuantity::powi`].

# Default values

The [Default] implementation creates a constant [`VarQuantity`] containing the
additive identity of the underlying quantity type. For example,
`VarQuantity::<ElectricalResistance>::default()` represents a resistance of 0 Ω.

# Serialization and deserialization

A serialized [`VarQuantity`] is represented as a sequence of [`RpnToken`]s.
Consequently, a [`VarQuantity`] can also be deserialized from such a sequence.
During deserialization, the token sequence is validated to ensure that it
represents a mathematically valid expression. For example, a [`RpnToken::Add`]
must have two operands. See also [`VarQuantity::from_tokens`].

```
use var_quantity::uom::si::{f64::ElectricalResistance, electrical_resistance::ohm};
use var_quantity::VarQuantity;

let string = indoc::indoc! {"
tokens:
- OperandConst:
    5 mOhm
- Clamp:
    min: 1.0
    max: 2.0
"};
let var_quantity: VarQuantity<ElectricalResistance> = serde_yaml::from_str(string).expect("valid sequence");
assert_eq!(var_quantity.eval(&[]).get::<ohm>(), 1.0);
```

In addition to the RPN representation, a [`VarQuantity`] can be deserialized from
several alternative representations:
- A quantity string, provided the `from_str` feature is enabled. The string is
  parsed as a constant [`DynQuantity`].
- The serialized representation of a [`QuantityFunction`] implementation,
  creating a variable quantity.
- A plain f64, creating a constant quantity using the unit specified by the
  target type. This mirrors the deserialization behaviour of uom.

```
use var_quantity::uom::si::{f64::ElectricalResistance, electrical_resistance::ohm};
use var_quantity::VarQuantity;

// From a string
let from_string: VarQuantity<ElectricalResistance> = serde_yaml::from_str("5.0 kOhm").unwrap();
assert_eq!(from_string.eval(&[]).get::<ohm>(), 5000.0);
assert!(serde_yaml::from_str::<VarQuantity<ElectricalResistance>>("5.0 m/s").is_err()); // Invalid output unit

// From a QuantityFunction implementor
let qf_string = indoc::indoc! {"
Polynomial:
  coefficients:
  - 5.0 kOhm
"};
let from_fn: VarQuantity<ElectricalResistance> = serde_yaml::from_str(&qf_string).unwrap();
assert_eq!(from_fn.eval(&[]).get::<ohm>(), 5000.0);

// From a f64
let from_f64: VarQuantity<ElectricalResistance> = serde_yaml::from_str("5000.0").unwrap();
assert_eq!(from_f64.eval(&[]).get::<ohm>(), 5000.0);
```
*/
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct VarQuantity<T> {
    tokens: Vec<RpnToken>,
    #[cfg_attr(feature = "serde", serde(skip))]
    max_stack_depth: usize,
    #[cfg_attr(feature = "serde", serde(skip))]
    function_count: usize,
    #[cfg_attr(feature = "serde", serde(skip))]
    _marker: PhantomData<T>,
}

impl<T: QuantityF64> VarQuantity<T> {
    /**
    Creates a new [`VarQuantity`] from the given [`QuantityFunction`]
    implementation if its output can be converted into `T`.

    During construction, `function` is evaluated with an empty `conditions`
    slice using [`QuantityFunction::eval`]. The returned [`DynQuantity`] is then
    checked for compatibility with `T`. As described in the [`QuantityFunction`]
    trait documentation, this check assumes that the function always returns
    quantities with the same unit, regardless of the supplied conditions.

    If the check succeeds, the resulting [`VarQuantity`] stores the given
    `function`. Subsequent calls to [`VarQuantity::eval`] evaluate this function
    with the supplied conditions.

    # Examples

    ```
    use dyn_quantity::{DynQuantity, PredefUnit};
    use var_quantity::{*, unary::Polynomial};
    use var_quantity::uom::si::f64::*;

    let function = Polynomial::new(vec![DynQuantity::new(1.0, PredefUnit::ElectricalResistance)]).expect("valid polynomial coefficients");

    // Output type of function can be converted into ElectricalResistance.
    assert!(VarQuantity::<ElectricalResistance>::new(function.clone()).is_ok());

    // Output type of function cannot be converted into Length.
    assert!(VarQuantity::<Length>::new(function.clone()).is_err());
    ```
    */
    pub fn new<F: QuantityFunction>(function: F) -> Result<Self, UnitsNotEqual> {
        // Check if the function produced the correct type for "default"
        // conditions.
        let output = function.eval(&[]);
        if T::try_from(output).is_err() {
            return Err(UnitsNotEqual(T::unit_from_type(), output.unit));
        }

        // Cast to trait object
        let boxed: Box<dyn QuantityFunction> = Box::new(function);

        Ok(Self {
            max_stack_depth: 1,
            function_count: 1,
            tokens: vec![boxed.into()],
            _marker: PhantomData,
        })
    }

    /**
    Fallibly creates a [`VarQuantity`] from a vector of [`RpnToken`]s.

    The token sequence must represent a valid arithmetic expression in
    Reverse Polish Notation (RPN). In particular, every operator must have
    the required number of operands, and the sequence must produce exactly
    one result.

    The expression is evaluated during construction to validate its
    structure and the compatibility of the operands' units. If evaluation
    fails, an [`EvalError`] is returned.

    This function performs token deduplication, see the documentation of
    [`VarQuantity`] and [`RpnToken::OperandFnRef`] for details.

    Consider building a [`VarQuantity`] directly using arithmetic operations
    instead (`+`, `-`, `*`, `/`) unless you specifically need to construct
    an expression from tokens.

    # Examples

    Comparison of token-based construction with direct construction using
    arithmetic operations, including examples of invalid token sequences.

    ```
    use var_quantity::*;
    use var_quantity::uom::si::f64::*;

    // Represents the valid sequence `1 + 2 - 3`
    let valid_sequence = vec![
       RpnToken::OperandConst(1.0.into()),
       RpnToken::OperandConst(2.0.into()),
       RpnToken::Add,
       RpnToken::OperandConst(3.0.into()),
       RpnToken::Sub,
    ];
    let expression: VarQuantity<f64> = VarQuantity::from_tokens(valid_sequence).expect("valid");
    assert_eq!(expression.eval(&[]), 0.0);

    // Alternative way to build the same expression:
    let expression: VarQuantity<f64> = VarQuantity::from(1.0) + VarQuantity::from(2.0) - VarQuantity::from(3.0);
    assert_eq!(expression.eval(&[]), 0.0);

    // Represents the invalid sequence `+ 2 - 3`
    let invalid_sequence = vec![
       RpnToken::OperandConst(2.0.into()),
       RpnToken::Add,
       RpnToken::OperandConst(3.0.into()),
       RpnToken::Sub,
    ];
    assert!(VarQuantity::<f64>::from_tokens(invalid_sequence).is_err());

    // Represents the invalid sequence `5 1 + 2 - 3`
    let invalid_sequence = vec![
       RpnToken::OperandConst(5.0.into()),
       RpnToken::OperandConst(1.0.into()),
       RpnToken::OperandConst(2.0.into()),
       RpnToken::Add,
       RpnToken::OperandConst(3.0.into()),
       RpnToken::Sub,
    ];
    assert!(VarQuantity::<f64>::from_tokens(invalid_sequence).is_err());
    ```
    */
    pub fn from_tokens(mut tokens: Vec<RpnToken>) -> Result<Self, EvalError> {
        for i in 0..tokens.len() {
            let (previous, current) = tokens.split_at_mut(i);
            deduplicate_fn_tokens(previous, &mut current[0]);
        }

        // Preanalyse the token stream to extract information for the storage
        // vector used for building the stack. If the preanalysis shows the
        // token stream is invalid, we can short-circuit here.
        let token_analysis = analyse_tokens(&tokens)?;
        let max_stack_depth = token_analysis.max_stack_depth;
        let function_count = token_analysis.function_count;

        let this = VarQuantity {
            tokens,
            max_stack_depth,
            function_count,
            _marker: PhantomData,
        };

        this.eval_checked(&[])?;
        Ok(this)
    }

    /**
    Clamps the values of `self` to the inclusive range [`min`, `max`], returning
    the resulting [`VarQuantity`].

    # Panics

    Panics if `min > max`, `min` is NaN, or `max` is NaN. This mirrors the
    behaviour of the standard library [`f64::clamp`] function.

    # Examples

    ```
    use var_quantity::*;

    let expression: VarQuantity<f64> = VarQuantity::from(1.0);
    assert_eq!(expression.eval(&[]), 1.0);

    assert_eq!(expression.clamp(2.0, 3.0).eval(&[]), 2.0);
    ```
    */
    pub fn clamp(mut self, min: T, max: T) -> Self {
        let ll: DynQuantity<f64> = min.into();
        let ul: DynQuantity<f64> = max.into();
        if ll.value > ul.value {
            panic!("min must not be larger than max");
        }
        if ll.value.is_nan() {
            panic!("min is NaN");
        }
        if ul.value.is_nan() {
            panic!("max is NaN");
        }

        self.tokens.push(RpnToken::Clamp {
            min: ll.value,
            max: ul.value,
        });
        self
    }

    /**
    Returns the absolute of `self`.

    # Examples

    ```
    use var_quantity::*;

    let absolute = VarQuantity::from(-2.0).abs();
    assert_eq!(absolute.eval(&[]), 2.0);

    let composed = absolute - VarQuantity::from(5.0);
    assert_eq!(composed.eval(&[]), -3.0);
    ```
    */
    pub fn abs(mut self) -> Self {
        self.tokens.push(RpnToken::Abs);
        self
    }

    /// Returns the underlying RPN token sequence for inspection.
    pub fn tokens(&self) -> &[RpnToken] {
        &self.tokens
    }

    /**
    Evaluates the expression for the given set of environmental `conditions`.

    If `self` contains one or more [`QuantityFunction`]s, their corresponding
    implementations determine which conditions it uses and how missing
    conditions are handled.

    During evaluation, repeated occurrences of equivalent [`QuantityFunction`]
    instances are deduplicated to improve performance, see struct documentation.
    Consequently, each function is evaluated at most once per evaluation. The
    equivalence of function instances is determined by
    [`QuantityFunction::dyn_eq`]. Implementations whose evaluation results
    depend on their internal state must ensure that instances with potentially
    different results do not compare equal.

    See [`VarQuantity`] for an example.
    */
    pub fn eval(&self, conditions: &[DynQuantity<f64>]) -> T {
        self.eval_priv::<false>(conditions).unwrap_or_default()
    }

    /**
    Like [`VarQuantity::eval`], but returns an [`EvalError`] if evaluation
    fails, for example because the evaluated quantity cannot be converted into
    `T`.

    This function is used by [`VarQuantity::from_tokens`] to validate the token
    sequence during construction. It is also exposed as public API for
    debugging, particularly when investigating suspected violations of the
    [`QuantityFunction`] trait contract, such as implementations returning
    different units for different inputs, or bugs in [`VarQuantity::eval`].

    Note that successful evaluation only establishes compatibility for the
    supplied conditions; it does not guarantee that the underlying
    [`QuantityFunction`] implementations return consistent units for all
    possible conditions.

    Prefer [`VarQuantity::eval`] for normal usage, as it is more performant.

    # Examples

    This example shows a [`QuantityFunction`] implementation that violates
    the trait contract by returning different units for different inputs.
    The unchecked [`VarQuantity::eval`] returns `T::default()` when the
    result cannot be converted into `T`, whereas [`VarQuantity::eval_checked`]
    returns an error.

    ```
    use var_quantity::*;
    use uom::si::{f64::Length, length::meter};

    #[derive(Clone, serde::Deserialize, serde::Serialize, PartialEq)]
    struct InvalidModel;

    // Invalid implementation of QuantityFunction that returns different units for different inputs.
    #[typetag::serde]
    impl QuantityFunction for InvalidModel {
        fn eval(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64> {
            if conditions.is_empty() {
                return DynQuantity::new(0.0, PredefUnit::Length);
            }
            DynQuantity::new(0.0, PredefUnit::Area)
        }

        fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
            (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
        }
    }

    // Construct a VarQuantity with the invalid model
    let var_quantity: VarQuantity<Length> = VarQuantity::new(InvalidModel {}).expect("output unit matches for no conditions");

    let conditions = &[DynQuantity::new(20.0, PredefUnit::Temperature)];

    // Default eval function silently casts.
    assert_eq!(var_quantity.eval(&[]).get::<meter>(), 0.0);
    assert_eq!(var_quantity.eval(conditions).get::<meter>(), 0.0);

    // Cecked eval function returns an error due to type mismatch
    assert_eq!(var_quantity.eval_checked(&[]).unwrap().get::<meter>(), 0.0);
    assert!(var_quantity.eval_checked(conditions).is_err());
    ```
     */
    pub fn eval_checked(&self, conditions: &[DynQuantity<f64>]) -> Result<T, EvalError> {
        self.eval_priv::<true>(conditions)
    }

    fn eval_priv<const CHECKED: bool>(
        &self,
        conditions: &[DynQuantity<f64>],
    ) -> Result<T, EvalError> {
        // Fast path for the common case that there is exactly one token
        // containing an operand. This avoids allocating the storage vector from
        // further below.
        if self.tokens.len() == 1 {
            match &self.tokens[0] {
                RpnToken::OperandFn(fun) => {
                    let dq = fun.eval(conditions);
                    return dq.try_into().map_err(|_| EvalError::UnitsNotEqual {
                        token_index: 0,
                        source: UnitsNotEqual(T::unit_from_type(), dq.unit),
                    });
                }
                RpnToken::OperandConst(value) => {
                    return (*value).try_into().map_err(|_| EvalError::UnitsNotEqual {
                        token_index: 0,
                        source: UnitsNotEqual(T::unit_from_type(), value.unit),
                    });
                }
                RpnToken::OperandFnRef(function_index) => {
                    return Err(EvalError::InvalidFunctionReference {
                        token_index: 0,
                        function_index: *function_index,
                    });
                }
                RpnToken::Clamp { min: _, max: _ } | RpnToken::Power(_) | RpnToken::Abs => {
                    return Err(EvalError::MissingOperand {
                        token_index: 0,
                        operator: "unary",
                        required: 1,
                        available: 0,
                    });
                }
                RpnToken::Add | RpnToken::Sub | RpnToken::Mul | RpnToken::Div => {
                    return Err(EvalError::MissingOperand {
                        token_index: 0,
                        operator: "binary",
                        required: 2,
                        available: 0,
                    });
                }
            }
        }

        /*
        This vector combines two logically separate storage regions into a single
        allocation:
        1) The first region [0..function_count] is a cache for function values,
           allowing them to be reused by later [`RpnToken::OperandFnRef`]s.
        2) The second region is the evaluation stack for the [`RpnToken`] stream
           in self.tokens.

        Since both the number of function values and the maximum stack depth are
        precalculated by [`analyse_tokens`], the required storage size is known
        beforehand and the vector only needs to be allocated once.
        */
        let mut storage: smallvec::SmallVec<[Option<DynQuantity<f64>>; 16]> =
            smallvec::smallvec![None; self.function_count + self.max_stack_depth];

        // Number of items on the stack. Add the function count to get the storage
        // index of the current stack item: storage[function_count + depth] is
        // the head of the stack.
        let mut depth = 0;

        // Number of evaluated functions. This is increased whenever we
        // encounter a new [`RpnToken::OperandFn`] and it will never exceed
        // function_count. It tells the algorithm the next free slot in storage
        // for caching function call results so later [`RpnToken::OperandFnRef`]
        // can access them.
        let mut function_cache_len: usize = 0;

        /*
        The evaluation loop implements a simple stack machine operating on the
        Reverse Polish Notation (RPN) instruction stream.
         */
        for (token_index, token) in self.tokens.iter().enumerate() {
            match token {
                RpnToken::OperandFn(fun) => {
                    let value = fun.eval(conditions);
                    storage[function_cache_len] = Some(value);
                    storage[self.function_count + depth] = Some(value);
                    depth += 1;
                    function_cache_len += 1;
                }
                RpnToken::OperandFnRef(function_index) => {
                    let value = match &storage[*function_index] {
                        Some(v) => *v,
                        None => {
                            return Err(EvalError::InvalidFunctionReference {
                                token_index,
                                function_index: *function_index,
                            });
                        }
                    };

                    storage[self.function_count + depth] = Some(value);
                    depth += 1;
                }
                RpnToken::OperandConst(value) => {
                    storage[self.function_count + depth] = Some(*value);
                    depth += 1;
                }
                RpnToken::Clamp { min, max } => {
                    if CHECKED && depth < 1 {
                        return Err(EvalError::MissingOperand {
                            token_index,
                            operator: "unary",
                            required: 1,
                            available: depth,
                        });
                    }

                    let stack_index = self.function_count + depth - 1;
                    let mut operand =
                        storage[stack_index]
                            .take()
                            .ok_or(EvalError::MissingStackValue {
                                token_index,
                                stack_index,
                            })?;
                    operand.value = operand.value.clamp(*min, *max);
                    storage[stack_index] = Some(operand);
                }
                RpnToken::Power(n) => {
                    if CHECKED && depth < 1 {
                        return Err(EvalError::MissingOperand {
                            token_index,
                            operator: "unary",
                            required: 1,
                            available: depth,
                        });
                    }

                    let stack_index = self.function_count + depth - 1;
                    let operand =
                        storage[stack_index]
                            .take()
                            .ok_or(EvalError::MissingStackValue {
                                token_index,
                                stack_index,
                            })?;
                    storage[stack_index] = Some(operand.powi(*n));
                }
                RpnToken::Abs => {
                    if CHECKED && depth < 1 {
                        return Err(EvalError::MissingOperand {
                            token_index,
                            operator: "unary",
                            required: 1,
                            available: depth,
                        });
                    }

                    let stack_index = self.function_count + depth - 1;
                    let operand =
                        storage[stack_index]
                            .take()
                            .ok_or(EvalError::MissingStackValue {
                                token_index,
                                stack_index,
                            })?;
                    storage[stack_index] = Some(operand.abs());
                }
                RpnToken::Add | RpnToken::Sub | RpnToken::Mul | RpnToken::Div => {
                    if CHECKED && depth < 2 {
                        return Err(EvalError::MissingOperand {
                            token_index,
                            operator: "binary",
                            required: 2,
                            available: depth,
                        });
                    }

                    let rhs_index = self.function_count + depth - 1;
                    let lhs_index = self.function_count + depth - 2;

                    let rhs = storage[rhs_index]
                        .take()
                        .ok_or(EvalError::MissingStackValue {
                            token_index,
                            stack_index: rhs_index,
                        })?;

                    let mut lhs =
                        storage[lhs_index]
                            .take()
                            .ok_or(EvalError::MissingStackValue {
                                token_index,
                                stack_index: lhs_index,
                            })?;

                    let result = match token {
                        RpnToken::Add => {
                            if CHECKED {
                                lhs.try_add(&rhs)
                                    .map_err(|source| EvalError::UnitsNotEqual {
                                        token_index,
                                        source,
                                    })?
                            } else {
                                lhs.value += rhs.value;
                                lhs
                            }
                        }
                        RpnToken::Sub => {
                            if CHECKED {
                                lhs.try_sub(&rhs)
                                    .map_err(|source| EvalError::UnitsNotEqual {
                                        token_index,
                                        source,
                                    })?
                            } else {
                                lhs.value -= rhs.value;
                                lhs
                            }
                        }
                        RpnToken::Mul => lhs * rhs,
                        RpnToken::Div => lhs / rhs,
                        _ => unreachable!(),
                    };

                    depth -= 1;
                    storage[lhs_index] = Some(result);
                }
            }
        }

        // At the end of the evaluation, the stack must only contain a single
        // item which we want to return.
        if CHECKED && depth != 1 {
            return Err(EvalError::InvalidStackDepth { depth });
        }

        let value = storage[self.function_count]
            .take()
            .ok_or(EvalError::MissingStackValue {
                token_index: self.tokens.len(),
                stack_index: self.function_count,
            })?;

        value.try_into().map_err(|_| EvalError::UnitsNotEqual {
            token_index: 0,
            source: UnitsNotEqual(T::unit_from_type(), value.unit),
        })
    }
}

impl<T: QuantityF64> Default for VarQuantity<T> {
    fn default() -> Self {
        Self::from(T::default())
    }
}

impl VarQuantity<f64> {
    /**
    Raises `self` to the integer power `n`, returning the resulting
    [`VarQuantity`].

    The exponent is restricted to `i8` to match the exponent representation
    used by [`DynQuantity`].

    # Examples

    ```
    use var_quantity::*;

    let expression: VarQuantity<f64> = VarQuantity::from(2.0) + VarQuantity::from(1.0);
    assert_eq!(expression.powi(3).eval(&[]), 27.0);
    ```
    */
    pub fn powi(mut self, n: i8) -> Self {
        self.tokens.push(RpnToken::Power(n));
        self
    }
}

impl<D> VarQuantity<uom::si::Quantity<D, uom::si::SI<f64>, f64>>
where
    D: uom::si::Dimension + ?Sized,
{
    /**
    Raises `self` to the integer power `n`, returning the resulting
    [`VarQuantity`].

    The exponent is restricted to `i8` to match the exponent representation
    used by [`DynQuantity`].

    # Examples

    ```
    use var_quantity::*;
    use uom::si::{f64::*, length::meter, volume::cubic_meter};
    use uom::typenum::P3;

    let expression: VarQuantity<Length> =
        VarQuantity::from(Length::new::<meter>(2.0)) + VarQuantity::from(Length::new::<meter>(1.0));
    assert_eq!(expression.powi(P3::new()).eval(&[]).get::<cubic_meter>(), 27.0);
    ```
    */
    pub fn powi<E>(
        mut self,
        _e: E,
    ) -> VarQuantity<
        uom::si::Quantity<
            uom::si::ISQ<
                Prod<D::L, E>,
                Prod<D::M, E>,
                Prod<D::T, E>,
                Prod<D::I, E>,
                Prod<D::Th, E>,
                Prod<D::N, E>,
                Prod<D::J, E>,
            >,
            uom::si::SI<f64>,
            f64,
        >,
    >
    where
        D::L: Mul<E>,
        <D::L as Mul<E>>::Output: Integer,
        D::M: Mul<E>,
        <D::M as Mul<E>>::Output: Integer,
        D::T: Mul<E>,
        <D::T as Mul<E>>::Output: Integer,
        D::I: Mul<E>,
        <D::I as Mul<E>>::Output: Integer,
        D::Th: Mul<E>,
        <D::Th as Mul<E>>::Output: Integer,
        D::N: Mul<E>,
        <D::N as Mul<E>>::Output: Integer,
        D::J: Mul<E>,
        <D::J as Mul<E>>::Output: Integer,
        D::Kind: uom::marker::Mul,
        E: Integer,
    {
        self.tokens.push(RpnToken::Power(E::to_i8()));
        VarQuantity {
            max_stack_depth: self.max_stack_depth,
            function_count: self.function_count,
            tokens: self.tokens,
            _marker: PhantomData,
        }
    }
}

impl<T: QuantityF64> PartialEq for VarQuantity<T> {
    fn eq(&self, other: &Self) -> bool {
        self.tokens == other.tokens
    }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for VarQuantity<T>
where
    T: serde::de::DeserializeOwned + QuantityF64,
    <T as TryFrom<DynQuantity<Complex<f64>>>>::Error: std::fmt::Display,
    <T as TryFrom<DynQuantity<f64>>>::Error: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct VarQuantityAlias {
            tokens: Vec<RpnToken>,
        }

        #[derive(deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError)]
        enum VarQuantityVariants<T> {
            VarQuantity(VarQuantityAlias),
            Quantity(T),
            QuantityFunction(Box<dyn QuantityFunction>),
            DynQuantity(DynQuantity<f64>),
        }

        let variants: VarQuantityVariants<T> = VarQuantityVariants::deserialize(deserializer)?;
        match variants {
            VarQuantityVariants::VarQuantity(alias) => {
                VarQuantity::from_tokens(alias.tokens).map_err(serde::de::Error::custom)
            }
            VarQuantityVariants::Quantity(qty) => Ok(VarQuantity::from(qty)),
            VarQuantityVariants::QuantityFunction(fun) => {
                VarQuantity::from_tokens(vec![RpnToken::OperandFn(fun)])
                    .map_err(serde::de::Error::custom)
            }
            VarQuantityVariants::DynQuantity(dyn_qty) => {
                let qty: T = dyn_qty.try_into().map_err(serde::de::Error::custom)?;
                Ok(VarQuantity::from(qty))
            }
        }
    }
}

impl<T: QuantityF64> From<T> for VarQuantity<T> {
    fn from(value: T) -> Self {
        Self {
            max_stack_depth: 1,
            function_count: 0,
            tokens: vec![RpnToken::OperandConst(value.into())],
            _marker: PhantomData,
        }
    }
}

impl<T> Add<VarQuantity<T>> for VarQuantity<T>
where
    T: QuantityF64,
{
    type Output = VarQuantity<T>;

    fn add(self, rhs: VarQuantity<T>) -> Self::Output {
        let mut tokens = self.tokens;

        tokens.reserve(rhs.tokens.len());
        for mut t in rhs.tokens {
            deduplicate_fn_tokens(tokens.as_slice(), &mut t);
            tokens.push(t);
        }

        tokens.push(RpnToken::Add);
        Self::Output {
            tokens,
            max_stack_depth: self.max_stack_depth + rhs.max_stack_depth,
            function_count: self.function_count + rhs.function_count,
            _marker: PhantomData,
        }
    }
}

impl<T> Sub<VarQuantity<T>> for VarQuantity<T>
where
    T: QuantityF64,
{
    type Output = VarQuantity<T>;

    fn sub(self, rhs: VarQuantity<T>) -> Self::Output {
        let mut tokens = self.tokens;

        tokens.reserve(rhs.tokens.len());
        for mut t in rhs.tokens {
            deduplicate_fn_tokens(tokens.as_slice(), &mut t);
            tokens.push(t);
        }

        tokens.push(RpnToken::Sub);
        Self::Output {
            tokens,
            max_stack_depth: self.max_stack_depth + rhs.max_stack_depth,
            function_count: self.function_count + rhs.function_count,
            _marker: PhantomData,
        }
    }
}

impl<Q, T, U> Mul<VarQuantity<Q>> for VarQuantity<T>
where
    T: QuantityF64 + Mul<Q, Output = U>,
    Q: QuantityF64,
    U: QuantityF64,
{
    type Output = VarQuantity<U>;

    fn mul(self, rhs: VarQuantity<Q>) -> Self::Output {
        let mut tokens = self.tokens;

        tokens.reserve(rhs.tokens.len());
        for mut t in rhs.tokens {
            deduplicate_fn_tokens(tokens.as_slice(), &mut t);
            tokens.push(t);
        }

        tokens.push(RpnToken::Mul);
        Self::Output {
            tokens,
            max_stack_depth: self.max_stack_depth + rhs.max_stack_depth,
            function_count: self.function_count + rhs.function_count,
            _marker: PhantomData,
        }
    }
}

impl<Q, T, U> Div<VarQuantity<Q>> for VarQuantity<T>
where
    T: QuantityF64 + Div<Q, Output = U>,
    Q: QuantityF64,
    U: QuantityF64,
{
    type Output = VarQuantity<U>;

    fn div(self, rhs: VarQuantity<Q>) -> Self::Output {
        let mut tokens = self.tokens;

        tokens.reserve(rhs.tokens.len());
        for mut t in rhs.tokens {
            deduplicate_fn_tokens(tokens.as_slice(), &mut t);
            tokens.push(t);
        }

        tokens.push(RpnToken::Div);
        Self::Output {
            tokens,
            max_stack_depth: self.max_stack_depth + rhs.max_stack_depth,
            function_count: self.function_count + rhs.function_count,
            _marker: PhantomData,
        }
    }
}

/**
A helper function which filters the `conditions` for a quantity with
the type `match_for`. If a matching quantity is found, it is used as argument
for `with_matched` and the result is returned. Otherwise, the result of
`no_match()` is returned.

The main purpose of this function is to simplify writing unary implementors of
[`QuantityFunction`]. For example, the [`QuantityFunction::eval`] implementation
of a linear function can look like this:

```
use dyn_quantity::{DynQuantity, Unit};
use var_quantity::{filter_unary_function, QuantityFunction};

// The serde annotations are just here because the doctests of this crate use
// the serde feature - they are not needed if the serde feature is disabled.
#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Linear {
    slope: f64,
    base_value: f64,
}

// Again, the macro annotation is just here because of the serde feature
#[cfg_attr(feature = "serde", typetag::serde)]
impl QuantityFunction for Linear {
    fn eval(&self, conditions: &[DynQuantity<f64>]) -> DynQuantity<f64> {
        return filter_unary_function(
            conditions,
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

    fn dyn_eq(&self, other: &dyn QuantityFunction) -> bool {
        (other as &dyn std::any::Any).downcast_ref::<Self>() == Some(self)
    }
}
```
 */
pub fn filter_unary_function<F, G>(
    conditions: &[DynQuantity<f64>],
    match_for: Unit,
    with_matched: F,
    no_match: G,
) -> DynQuantity<f64>
where
    F: FnOnce(DynQuantity<f64>) -> DynQuantity<f64>,
    G: FnOnce() -> DynQuantity<f64>,
{
    for iq in conditions {
        if iq.unit == match_for {
            return with_matched(*iq);
        }
    }
    no_match()
}
