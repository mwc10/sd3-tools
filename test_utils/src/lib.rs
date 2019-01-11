/* Utilities that are reused for testing */

/// Compare doubles `A` and `B` within percent tolerance `tol`
pub fn double_comparable(a: f64, b: f64, tol: f64) -> bool {
    if !a.is_finite() || !b.is_finite()  { return false; }
    
    let diff = (a-b).abs();
    let a = a.abs();
    let b = b.abs();
    let largest = a.max(b);
    
    if diff <= (largest * tol / 100.0)
    { true } else { false }
}