use std::fmt::{Debug, Error, Formatter};
use std::ops::{
    Add, AddAssign,
    Div, DivAssign,
    Index, IndexMut,
    Mul,
    MulAssign,
    Neg,
    Not, Rem,
    Sub, SubAssign,
};

#[derive(Copy, Clone)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn angle(&self) -> f64 {
        (self.y).atan2(self.x)
    }

    pub fn normalize(&mut self) -> &mut Vector2 {
        let norm = self.magnitude();
        self.x /= norm;
        self.y /= norm;

        self
    }

    pub fn rotation(&mut self, angle: f64) -> &mut Vector2 {
        let c = angle.cos();
        let s = angle.sin();

        self.x = self.x * c - self.y * s;
        self.y = self.x * s + self.y * c;

        self
    }

    pub fn as_array(&self) -> [f64; 2] {
        [self.x, self.y]
    }

    pub fn set_array(&mut self, array: &[f64; 2]) -> &mut Vector2 {
        self.x = array[0];
        self.y = array[1];

        self
    }

    pub fn new(x: f64, y: f64) -> Vector2 {
        Vector2 { x, y }
    }

    pub fn radial(mag: f64, ang: f64) -> Vector2 {
        let x = mag * ang.cos();
        let y = mag * ang.sin();

        Vector2 { x, y }
    }

    pub fn zeros() -> Vector2 {
        Vector2::new(0., 0.)
    }

    pub fn ones() -> Vector2 {
        Vector2::new(1., 1.)
    }

    pub fn ex() -> Vector2 {
        Vector2::new(1., 0.)
    }

    pub fn ey() -> Vector2 {
        Vector2::new(0., 1.)
    }
}

impl Debug for Vector2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if self.magnitude() > 1e6f64 {
            write!(f, "({:e}, {:e})", self.x, self.y)
        } else {
            write!(f, "({:.3}, {:.3})", self.x, self.y)
        }
    }
}

impl PartialEq for Vector2 {
    fn eq(&self, other: &Self) -> bool {
        (*self % *other) < std::f64::MIN_POSITIVE
    }

    fn ne(&self, other: &Self) -> bool {
        (*self % *other) >= std::f64::MIN_POSITIVE
    }
}

impl Add<Vector2> for Vector2 {
    type Output = Vector2;

    fn add(self, rhs: Vector2) -> Self::Output {
        Vector2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign<Vector2> for Vector2 {
    fn add_assign(&mut self, rhs: Vector2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub<Vector2> for Vector2 {
    type Output = Vector2;

    fn sub(self, rhs: Vector2) -> Self::Output {
        Vector2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign<Vector2> for Vector2 {
    fn sub_assign(&mut self, rhs: Vector2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Neg for Vector2 {
    type Output = Vector2;

    fn neg(self) -> Self::Output {
        Vector2::new(-self.x, -self.y)
    }
}

impl Mul<f64> for Vector2 {
    type Output = Vector2;

    fn mul(self, rhs: f64) -> Self::Output {
        Vector2::new(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign<f64> for Vector2 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div<f64> for Vector2 {
    type Output = Vector2;

    fn div(self, rhs: f64) -> Self::Output {
        Vector2::new(self.x / rhs, self.y / rhs)
    }
}

impl DivAssign<f64> for Vector2 {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Not for Vector2 {
    type Output = f64;

    fn not(self) -> Self::Output {
        self.magnitude()
    }
}

impl Rem<Vector2> for Vector2 {
    type Output = f64;

    fn rem(self, rhs: Vector2) -> Self::Output {
        let dx = self.x - rhs.x;
        let dy = self.y - rhs.y;

        (dx * dx + dy * dy).sqrt()
    }
}

impl Mul<Vector2> for Vector2 {
    type Output = f64;

    fn mul(self, rhs: Vector2) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y
    }
}

impl From<[f64; 2]> for Vector2 {
    fn from(array: [f64; 2]) -> Self {
        Vector2::new(array[0], array[1])
    }
}

impl Index<usize> for Vector2 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        if index == 0 {
            &self.x
        } else {
            &self.y
        }
    }
}

impl IndexMut<usize> for Vector2 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index == 0 {
            &mut self.x
        } else {
            &mut self.y
        }
    }
}

#[cfg(test)]
mod tests {
    mod vector2 {
        use super::super::Vector2;

        #[test]
        fn norm_vector() {
            let u = Vector2::new(-4., 0.);

            assert_eq!(!u, 4.);
            assert_eq!(u % u, 0.);
        }

        #[test]
        fn polar_coordinates() {
            let u = Vector2::ones();

            assert_eq!(u.magnitude(), std::f64::consts::SQRT_2);
            assert_eq!(u.angle(), std::f64::consts::FRAC_PI_4);
        }

        #[test]
        fn partial_eq_vector() {
            let u = Vector2::new(-4., 0.);
            let v = Vector2::new(-2., 0.);

            assert_eq!(u, u);
            assert_ne!(u, v);
        }

        #[test]
        fn arithmetic_vector() {
            let mut u = Vector2::new(-4., 1.);
            let v = Vector2::new(3., 2.);

            assert_eq!(u + v, Vector2::new(-1., 3.));
            assert_eq!(u - v, Vector2::new(-7., -1.));
            assert_eq!(u + v, Vector2::new(-1., 3.));
            assert_eq!(u * 2., Vector2::new(-8., 2.));
            assert_eq!(u / 4., Vector2::new(-1., 0.25));

            u += v;
            assert_eq!(u, Vector2::new(-1., 3.));
        }
    }
}