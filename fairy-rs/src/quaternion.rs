//! Pure quaternion operations for multiplication-free computation

use num_complex::Complex32;

/// Quaternion group operations
pub struct QuaternionOps;

impl QuaternionOps {
    /// Group multiplication table for Q_4 = {±1, ±i}
    pub fn multiply_table() -> [[Complex32; 4]; 4] {
        use crate::QUATERNIONS;
        let mut table = [[Complex32::new(0.0, 0.0); 4]; 4];
        
        // Build multiplication table
        // This is precomputed at compile time!
        for i in 0..4 {
            for j in 0..4 {
                table[i][j] = QUATERNIONS[i] * QUATERNIONS[j];
            }
        }
        
        table
    }
}