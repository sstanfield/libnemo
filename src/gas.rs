use std::cmp::Ordering;
use std::fmt;

use crate::segment_type::SegmentType;
use crate::types::*;

#[derive(Copy, Clone)]
pub struct Gas {
    pub f_o2: f64,
    pub f_n2: f64,
    pub f_he: f64,
    pub min_ppo2: f64,
    pub ppo2: f64,
    min_depth: Pressure,
    max_depth: Pressure,
    pub use_ascent: bool,
    pub use_descent: bool,
    pub use_diluent: bool,
}

//pub const AIR: Gas = Gas::new_bottom(0.21, 0.0, 1.4);

impl Gas {
    fn new(
        f_o2: f64,
        f_he: f64,
        ppo2: f64,
        min_ppo2: f64,
        use_ascent: bool,
        use_descent: bool,
        use_diluent: bool,
    ) -> Gas {
        let min = Pressure::bar(if f_o2 >= min_ppo2 {
            1.0
        } else {
            min_ppo2 / f_o2
        });
        let max = Pressure::bar(ppo2 / f_o2);
        Gas {
            f_o2,
            f_n2: 1.0 - (f_o2 + f_he),
            f_he,
            min_ppo2,
            ppo2,
            min_depth: min,
            max_depth: max,
            use_ascent,
            use_descent,
            use_diluent,
        }
    }
    pub fn new_deco(f_o2: f64, f_he: f64) -> Gas {
        Gas::new(f_o2, f_he, 1.61, 0.21, true, false, false)
    }
    pub fn new_bottom(f_o2: f64, f_he: f64, ppo2: f64) -> Gas {
        Gas::new(f_o2, f_he, ppo2, 0.18, true, true, false)
    }
    pub fn new_diluent(f_o2: f64, f_he: f64) -> Gas {
        Gas::new(f_o2, f_he, 1.61, 0.18, false, false, true)
    }

    pub fn use_gas(&self, depth: Pressure, segment: SegmentType) -> bool {
        if depth >= self.min_depth && depth <= self.max_depth {
            match segment {
                SegmentType::DOWN if self.use_descent => return true,
                SegmentType::UP if self.use_ascent => return true,
                SegmentType::LEVEL => return true,
                _ => return false,
            }
        }
        return false;
    }

    //int get hashCode => (f_o2 * 1000 + f_he * 1000).ceil();
}

impl PartialEq for Gas {
    fn eq(&self, other: &Gas) -> bool {
        (self.f_o2 * 1000.0) as u32 == (other.f_o2 * 1000.0) as u32
            && (self.f_he * 1000.0) as u32 == (other.f_he * 1000.0) as u32
    }
}
impl Eq for Gas {}
impl PartialOrd for Gas {
    fn partial_cmp(&self, other: &Gas) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Gas {
    fn cmp(&self, other: &Gas) -> Ordering {
        if self.eq(other) {
            return Ordering::Equal;
        }
        if self.f_o2 < other.f_o2 {
            return Ordering::Less;
        }
        if (self.f_o2 * 1000.0) as u32 == (other.f_o2 * 1000.0) as u32 && self.f_he < other.f_he {
            return Ordering::Less;
        }
        Ordering::Greater
    }
}
impl fmt::Display for Gas {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.f_he > 0.0 {
            write!(
                f,
                "{}/{}",
                ((self.f_o2 * 100.0).round()) as i32,
                ((self.f_he * 100.0).round()) as i32
            )
        } else {
            write!(f, "{}%", ((self.f_o2 * 100.0).round()) as i32)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn display() {
        assert_eq!(
            "21%".to_owned(),
            format!("{}", Gas::new_bottom(0.21, 0.0, 1.4))
        );
        assert_eq!(
            "18/45".to_owned(),
            format!("{}", Gas::new_bottom(0.18, 0.45, 1.4))
        );
    }
}
