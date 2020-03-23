use std::arch::x86_64::*;

/// # Safety
/// requires sse
/// compute the dot for four vectors at once
/// i.e.
/// a_x contains all x components of the 4 a vectors
/// b_x contains all x components of the 4 b vectors
/// sum[0] contains the dot product for the first vector, sum[1] for the second etc.
pub unsafe fn sse_dot_product(
    a_x: __m128,
    a_y: __m128,
    a_z: __m128,
    b_x: __m128,
    b_y: __m128,
    b_z: __m128,
) -> __m128 {
    let a_x = _mm_mul_ps(a_x, b_x);
    let a_y = _mm_mul_ps(a_y, b_y);
    let a_z = _mm_mul_ps(a_z, b_z);
    _mm_add_ps(_mm_add_ps(a_x, a_y), a_z)
}
/// # Safety
/// requires sse
/// compute the cross product of four vectors at once
/// i.e.
/// a_x contains all x components of the 4 a vectors
/// b_x contains all x components of the 4 b vectors
/// c_x contains the x components of the cross product
/// i.e. c_x[0] is the x component of the cross product of a_*[0] and b_*[0]
pub unsafe fn sse_cross_product(
    a_x: __m128,
    a_y: __m128,
    a_z: __m128,
    b_x: __m128,
    b_y: __m128,
    b_z: __m128,
) -> (__m128, __m128, __m128) {
    let c_x = _mm_sub_ps(_mm_mul_ps(a_y, b_z), _mm_mul_ps(a_z, b_y));
    let c_y = _mm_sub_ps(_mm_mul_ps(a_z, b_x), _mm_mul_ps(a_x, b_z));
    let c_z = _mm_sub_ps(_mm_mul_ps(a_x, b_y), _mm_mul_ps(a_y, b_x));

    (c_x, c_y, c_z)
}

#[cfg(test)]
mod tests {
    use super::{sse_cross_product, sse_dot_product};
    use std::arch::x86_64::*;
    use std::mem;

    unsafe fn assert_m128_equal(a: __m128, b: __m128) {
        let a: [f32; 4] = mem::transmute(a);
        let b: [f32; 4] = mem::transmute(b);
        assert_eq!(a, b);
    }

    #[test]
    fn test_sse_cross_product() {
        if is_x86_feature_detected!("sse") {
            unsafe {
                let a_x = _mm_loadu_ps(vec![1.0, 0.0, 3.0, 2.0].as_ptr());
                let a_y = _mm_loadu_ps(vec![0.0, 1.0, 4.0, 6.0].as_ptr());
                let a_z = _mm_loadu_ps(vec![0.0, 0.0, 4.0, 3.0].as_ptr());

                let b_x = _mm_loadu_ps(vec![0.0, 0.0, 1.0, 2.0].as_ptr());
                let b_y = _mm_loadu_ps(vec![1.0, 0.0, -2.0, 1.0].as_ptr());
                let b_z = _mm_loadu_ps(vec![0.0, 1.0, 3.0, -2.0].as_ptr());

                let c_x_exp = _mm_loadu_ps(vec![0.0, 1.0, 20.0, -15.0].as_ptr());
                let c_y_exp = _mm_loadu_ps(vec![0.0, 0.0, -5.0, 10.0].as_ptr());
                let c_z_exp = _mm_loadu_ps(vec![1.0, 0.0, -10.0, -10.0].as_ptr());

                let (c_x, c_y, c_z) = sse_cross_product(a_x, a_y, a_z, b_x, b_y, b_z);

                assert_m128_equal(c_x, c_x_exp);
                assert_m128_equal(c_y, c_y_exp);
                assert_m128_equal(c_z, c_z_exp);
            }
        } else {
            println!("test_sse_cross_product() could not be run, because CPU does not support it!");
        }
    }

    #[test]
    fn test_trivial_sse_cross_product() {
        if is_x86_feature_detected!("sse") {
            unsafe {
                let a_x = _mm_set1_ps(0.0);
                let a_y = _mm_set1_ps(0.0);
                let a_z = _mm_set1_ps(1.0);

                let b_x = _mm_set1_ps(1.0);
                let b_y = _mm_set1_ps(0.0);
                let b_z = _mm_set1_ps(0.0);

                let c_x_exp = _mm_set1_ps(0.0);
                let c_y_exp = _mm_set1_ps(1.0);
                let c_z_exp = _mm_set1_ps(0.0);

                let (c_x, c_y, c_z) = sse_cross_product(a_x, a_y, a_z, b_x, b_y, b_z);

                assert_m128_equal(c_x, c_x_exp);
                assert_m128_equal(c_y, c_y_exp);
                assert_m128_equal(c_z, c_z_exp);
            }
        } else {
            println!("test_trivial_sse_cross_product() could not be run, because CPU does not support it!");
        }
    }

    #[test]
    fn test_sse_dot_product() {
        if is_x86_feature_detected!("sse") {
            unsafe {
                let a_x = _mm_loadu_ps(vec![1.0, 0.0, 3.0, 2.0].as_ptr());
                let a_y = _mm_loadu_ps(vec![0.0, 1.0, 4.0, 6.0].as_ptr());
                let a_z = _mm_loadu_ps(vec![0.0, 0.0, 4.0, 3.0].as_ptr());

                let b_x = _mm_loadu_ps(vec![0.0, 0.0, 1.0, 2.0].as_ptr());
                let b_y = _mm_loadu_ps(vec![1.0, 0.0, -2.0, 1.0].as_ptr());
                let b_z = _mm_loadu_ps(vec![0.0, 1.0, 3.0, -2.0].as_ptr());

                let s_exp = _mm_loadu_ps(vec![0.0, 0.0, 7.0, 4.0].as_ptr());

                let s = sse_dot_product(a_x, a_y, a_z, b_x, b_y, b_z);

                assert_m128_equal(s, s_exp);
            }
        } else {
            println!("test_sse_dot_product() could not be run, because CPU does not support it!");
        }
    }

    #[test]
    fn test_trivial_sse_dot_product() {
        if is_x86_feature_detected!("sse") {
            unsafe {
                let a_x = _mm_set1_ps(0.0);
                let a_y = _mm_set1_ps(0.0);
                let a_z = _mm_set1_ps(1.0);

                let b_x = _mm_set1_ps(1.0);
                let b_y = _mm_set1_ps(0.0);
                let b_z = _mm_set1_ps(0.0);

                let s_exp = _mm_set1_ps(0.0);

                let s = sse_dot_product(a_x, a_y, a_z, b_x, b_y, b_z);

                assert_m128_equal(s, s_exp);

                let a_x = _mm_set1_ps(1.0);
                let a_y = _mm_set1_ps(1.0);
                let a_z = _mm_set1_ps(1.0);

                let b_x = _mm_set1_ps(1.0);
                let b_y = _mm_set1_ps(1.0);
                let b_z = _mm_set1_ps(1.0);

                let s_exp = _mm_set1_ps(3.0);

                let s = sse_dot_product(a_x, a_y, a_z, b_x, b_y, b_z);

                assert_m128_equal(s, s_exp);
            }
        } else {
            println!(
                "test_trivial_sse_dot_product() could not be run, because CPU does not support it!"
            );
        }
    }
}
