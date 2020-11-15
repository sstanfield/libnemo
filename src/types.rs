use std::cmp::Ordering;

// Use f64 to avoid casting for calculations.  Normalized to mm.
#[derive(Copy, Clone)]
pub struct Depth(f64);

// Use f64 to avoid casting for math.  Normalized to mbar.
#[derive(Copy, Clone)]
pub struct Pressure(f64);

// Use f64 to avoid casting for math.  Normalized to mbar/min.
// Rate of depth change for ascent and descent.
#[derive(Copy, Clone)]
pub struct DepthChange(f64);

impl Depth {
    pub fn millimeters(mm: f64) -> Depth {
        Depth(mm)
    }
    pub fn meters(meters: f64) -> Depth {
        Depth(meters * 1000.0)
    }
    pub fn feet(feet: f64) -> Depth {
        Depth(feet * 304.8)
    }
    pub fn from_pressure(pressure: Pressure, atm: Pressure) -> Depth {
        Depth((pressure.to_mbar() - atm.to_mbar()) * 10.0)
    }

    pub fn to_mm(&self) -> f64 {
        self.0
    }
    pub fn to_meters(&self) -> f64 {
        self.0 / 1000.0
    }
    pub fn to_feet(&self) -> f64 {
        self.0 / 304.8
    }
}
impl PartialEq for Depth {
    fn eq(&self, other: &Depth) -> bool {
        self.0 == other.0
    }
}
impl Eq for Depth {}
impl PartialOrd for Depth {
    fn partial_cmp(&self, other: &Depth) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Depth {
    fn cmp(&self, other: &Depth) -> Ordering {
        if self.eq(other) {
            return Ordering::Equal;
        }
        if self.0 < other.0 {
            return Ordering::Less;
        }
        Ordering::Greater
    }
}

impl Pressure {
    pub fn millibar(mbar: f64) -> Pressure {
        Pressure(mbar)
    }
    pub fn bar(bar: f64) -> Pressure {
        Pressure(bar * 1000.0)
    }
    pub fn from_depth(depth: Depth, atm: Pressure) -> Pressure {
        Pressure((depth.to_mm() / 10.0) + atm.to_mbar())
    }
    pub fn from_depth_rel(depth: Depth) -> Pressure {
        Pressure(depth.to_mm() / 10.0)
    }

    pub fn to_mbar(&self) -> f64 {
        self.0
    }
    pub fn to_bar(&self) -> f64 {
        self.0 / 1000.0
    }
    pub fn to_depth(&self, atm: Pressure) -> Depth {
        Depth::from_pressure(*self, atm)
    }
}
impl PartialEq for Pressure {
    fn eq(&self, other: &Pressure) -> bool {
        self.0 == other.0
    }
}
impl Eq for Pressure {}
impl PartialOrd for Pressure {
    fn partial_cmp(&self, other: &Pressure) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Pressure {
    fn cmp(&self, other: &Pressure) -> Ordering {
        if self.eq(other) {
            return Ordering::Equal;
        }
        if self.0 < other.0 {
            return Ordering::Less;
        }
        Ordering::Greater
    }
}

impl DepthChange {
    pub fn descent_pressure(mbar: Pressure) -> DepthChange {
        DepthChange(mbar.to_mbar())
    }
    pub fn descent_depth(depth: Depth) -> DepthChange {
        DepthChange(depth.to_mm() / 10.0)
    }
    pub fn ascent_pressure(mbar: Pressure) -> DepthChange {
        DepthChange(-mbar.to_mbar())
    }
    pub fn ascent_depth(depth: Depth) -> DepthChange {
        DepthChange(-depth.to_mm() / 10.0)
    }

    pub fn to_mbar(&self) -> f64 {
        self.0
    }
    pub fn to_bar(&self) -> f64 {
        self.0 / 1000.0
    }
}
impl PartialEq for DepthChange {
    fn eq(&self, other: &DepthChange) -> bool {
        self.0 == other.0
    }
}
impl Eq for DepthChange {}
impl PartialOrd for DepthChange {
    fn partial_cmp(&self, other: &DepthChange) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for DepthChange {
    fn cmp(&self, other: &DepthChange) -> Ordering {
        if self.eq(other) {
            return Ordering::Equal;
        }
        if self.0 < other.0 {
            return Ordering::Less;
        }
        Ordering::Greater
    }
}
