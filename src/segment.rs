use segment_type::SegmentType;
use gas::Gas;
use otu_cns::OtuCns;
use types::*;


#[derive(Copy,Clone)]
pub struct SegmentIn {
    pub segment_type: SegmentType,
    pub depth: Depth, // in mbar
    pub time: f64,
    pub setpoint: f64
}

impl SegmentIn {
    fn new(segment_type: SegmentType, depth: Depth, time: f64, setpoint: f64) -> SegmentIn {
        SegmentIn { segment_type, depth, time, setpoint }
    }
    pub fn new_bottom(depth: Depth, time: f64, setpoint: f64) -> SegmentIn {
        SegmentIn::new(SegmentType::LEVEL, depth, time, setpoint)
    }
}

pub struct Segment {
    pub segment_type: SegmentType,
    pub depth: Pressure, // in mbar
    pub raw_time: f64,
    pub time: u32,
    pub gas: Gas,
    pub ceiling: i32,
    pub otu_cns: OtuCns,
    pub setpoint: f64,
    pub compartments: Compartments,
}

/*impl Segment {
    pub fn new(segment_type: SegmentType, depth: Pressure, raw_time: f64, time: u32, gas: Gas,
           ceiling: i32, otu_cns: OtuCns, setpoint: Option<f64>) -> Segment {
        let tsetpoint = setpoint.unwrap_or(1.0);
        Segment { segment_type, depth, raw_time, time, gas, ceiling, otu_cns, setpoint: tsetpoint }
    }
}*/

pub struct Compartments {
    pub nitrogen: Vec<f64>,
    pub helium: Vec<f64>,
}

impl Compartments {
    pub fn new_surface(atm_pressure: f64, partial_water: f64, num_compartments: usize) -> Compartments {
        let mut nitrogen = Vec::with_capacity(num_compartments);
        let mut helium = Vec::with_capacity(num_compartments);
        let n2_partial = 0.79 * ((atm_pressure - partial_water) / 1000.0);
        for _i in 0 .. num_compartments {
            nitrogen.push(n2_partial);
            helium.push(0.0);
        }
        Compartments { nitrogen, helium }
    }

    pub fn new_empty(num_compartments: usize) -> Compartments {
        let mut nitrogen = Vec::with_capacity(num_compartments);
        let mut helium = Vec::with_capacity(num_compartments);
        nitrogen.resize(num_compartments, 0.0);
        helium.resize(num_compartments, 0.0);
        Compartments { nitrogen, helium }
    }

    pub fn new_copy(compartments: &Compartments) -> Compartments {
        let mut nitrogen = Vec::with_capacity(compartments.nitrogen.len());
        let mut helium = Vec::with_capacity(compartments.helium.len());
        for f in &compartments.nitrogen { nitrogen.push(*f); }
        for f in &compartments.helium { helium.push(*f); }
        Compartments { nitrogen, helium }
    }
}


