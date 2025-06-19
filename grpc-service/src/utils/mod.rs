pub mod macros;

use sqlx::types::Decimal;

pub fn to_f64(num: Decimal) -> f64 {
    let num_str = num.to_string();
    let num_f64: f64 = num_str.parse().unwrap();
    num_f64
}
