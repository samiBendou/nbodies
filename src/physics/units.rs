pub const PX_PER_METER: f64 = 1.;

pub mod prefix {
    pub struct Scale<'a> {
        pub label: &'a str,
        pub multiplier: f64,
    }

    pub enum Standard<'a> {
        Base { info: Scale<'a> },
        Kilo { info: Scale<'a> },
        Mega { info: Scale<'a> },
        Giga { info: Scale<'a> },
        Tera { info: Scale<'a> },
    }

    impl From<f64> for Scale<'static> {
        fn from(val: f64) -> Self {
            use Standard::*;
            match val.abs().log10().floor() as i32 {
                0 | 1 | 2 => Scale::new("", 1.),
                3 | 4 | 5 => Scale::new("k", 1e-3),
                6 | 7 | 8 => Scale::new("M", 1e-6),
                9 | 10 | 11 => Scale::new("G", 1e-9),
                12 | 13 | 14 => Scale::new("T", 1e-12),
                _ => Scale::new("", 1.)
            }
        }
    }

    impl Scale<'static> {
        pub fn new(label: &str, multiplier: f64) -> Scale {
            Scale { label, multiplier }
        }

        pub fn value_of(&self, val: f64) -> f64 {
            val * self.multiplier
        }
    }
}