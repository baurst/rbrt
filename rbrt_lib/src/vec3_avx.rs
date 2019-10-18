use std::arch::x86_64::*;

/// compute the dot for 8 vectors at once
/// i.e.
/// a_x contains all x components of the 8 a vectors
/// b_x contains all x components of the 8 b vectors
/// sum[0] contains the dot product for the first vector, sum[1] for the second etc.
pub unsafe fn avx_dot_product(
    a_x: __m256,
    a_y: __m256,
    a_z: __m256,
    b_x: __m256,
    b_y: __m256,
    b_z: __m256,
) -> __m256 {
    let a_x = _mm256_mul_ps(a_x, b_x);
    let a_y = _mm256_mul_ps(a_y, b_y);
    let a_z = _mm256_mul_ps(a_z, b_z);
    let sum = _mm256_add_ps(_mm256_add_ps(a_x, a_y), a_z);
    return sum;
}

/// compute the cross product of 8 vectors at once
/// i.e.
/// a_x contains all x components of the 8 a vectors
/// b_x contains all x components of the 8 b vectors
/// c_x contains the x components of the cross product
/// i.e. c_x[0] is the x component of the cross product of a_*[0] and b_*[0]
pub unsafe fn avx_cross_product(
    a_x: __m256,
    a_y: __m256,
    a_z: __m256,
    b_x: __m256,
    b_y: __m256,
    b_z: __m256,
) -> (__m256, __m256, __m256) {
    let c_x = _mm256_sub_ps(_mm256_mul_ps(a_y, b_z), _mm256_mul_ps(a_z, b_y));
    let c_y = _mm256_sub_ps(_mm256_mul_ps(a_z, b_x), _mm256_mul_ps(a_x, b_z));
    let c_z = _mm256_sub_ps(_mm256_mul_ps(a_x, b_y), _mm256_mul_ps(a_y, b_x));

    return (c_x, c_y, c_z);
}
