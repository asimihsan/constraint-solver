///! In order to test local search methods, use the Ackley Function [3] from [2].
///!
///! [2] Optimization Test Problems: https://www.sfu.ca/~ssurjano/optimization.html
///!
///! [3] Ackley Function: https://www.sfu.ca/~ssurjano/ackley.html
use float_ord::FloatOrd;
pub struct AckleyFunction {
    a: f64,
    b: f64,
    c: f64,
}

impl AckleyFunction {
    pub fn new(a: f64, b: f64, c: f64) -> Self {
        AckleyFunction { a, b, c }
    }

    pub fn calculate(&self, xs: &Vec<FloatOrd<f64>>) -> f64 {
        let dimensions: f64 = xs.len() as f64;
        let mut fx: f64 = 0.0;
        let mut square_sum = 0.0;
        let mut cosine_sum = 0.0;
        xs.into_iter().for_each(|xi| {
            square_sum += xi.0 * xi.0;
            cosine_sum += (self.c * xi.0).cos();
        });
        fx += -self.a * (-self.b * (square_sum / dimensions).sqrt()).exp();
        fx -= (cosine_sum / dimensions).exp();
        fx += self.a + std::f64::consts::E;
        fx
    }
}

impl Default for AckleyFunction {
    fn default() -> Self {
        let a = 20.0;
        let b = 0.2;
        let c = 2.0 * std::f64::consts::PI;
        Self::new(a, b, c)
    }
}

/// Ackley MATLAB implementation: https://www.sfu.ca/~ssurjano/Code/ackleym.html
/// Copy/pasted it into Octave, than ran for some few examples.
///
/// Here is how to call Octave functions (note floating point errors):
///
/// octave:1> format long
/// octave:2> ackley([0, 0], 20.0, 0.2, 2 * pi)
/// ans = 4.440892098500626e-16
/// octave:3> ackley([1.0, 1.0], 20.0, 0.2, 2 * pi)
/// ans = 3.625384938440363
#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use float_ord::FloatOrd;

    use super::AckleyFunction;

    #[test]
    fn test_ackley_function_zero() {
        let ackley = AckleyFunction::default();
        let actual_result = ackley.calculate(&vec![FloatOrd(0.0), FloatOrd(0.0)]);
        assert_abs_diff_eq!(0.0, actual_result, epsilon = 1e-12);
    }

    #[test]
    fn test_ackley_function_2d() {
        let ackley = AckleyFunction::default();
        let actual_result = ackley.calculate(&vec![FloatOrd(1.0), FloatOrd(1.0)]);
        assert_abs_diff_eq!(3.625384938440363, actual_result, epsilon = 1e-12);
    }

    #[test]
    fn test_ackley_function_20d() {
        let ackley = AckleyFunction::default();
        let actual_result = ackley.calculate(&vec![
            FloatOrd(0.0),
            FloatOrd(1.0),
            FloatOrd(2.0),
            FloatOrd(3.0),
            FloatOrd(4.0),
            FloatOrd(5.0),
            FloatOrd(6.0),
            FloatOrd(7.0),
            FloatOrd(8.0),
            FloatOrd(9.0),
            FloatOrd(0.0),
            FloatOrd(1.0),
            FloatOrd(2.0),
            FloatOrd(3.0),
            FloatOrd(4.0),
            FloatOrd(5.0),
            FloatOrd(6.0),
            FloatOrd(7.0),
            FloatOrd(8.0),
            FloatOrd(9.0),
        ]);
        assert_abs_diff_eq!(13.12408690638194, actual_result, epsilon = 1e-12);
    }
}
