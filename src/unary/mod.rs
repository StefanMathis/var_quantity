/*!
This module contains unary functions which implement [`QuantityFunction`](crate::QuantityFunction).
*/

pub mod exponential;
pub mod first_order_taylor;
pub mod linear;
pub mod polynomial;

pub use exponential::{ExpTerm, Exponential};
pub use first_order_taylor::FirstOrderTaylor;
pub use linear::Linear;
pub use polynomial::Polynomial;
