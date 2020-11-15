use dive_consts::*;
use gas::Gas;
use segment::*;
use segment_type::SegmentType;
use types::*;

//use otu_cns::OtuCns;

#[derive(Copy, Clone)]
pub enum DiveType {
    OC,
    CCR,
    //    SCR
}

pub const CONSTANTS_A: TissueConstants = TissueConstants {
    half_times_n2: &HALF_TIMES_N2,
    half_times_he: &HALF_TIMES_HE,
    he_a_s: &HE_AS,
    he_b_s: &HE_BS,
    n2_a_s: &N2AS_A,
    n2_b_s: &N2BS,
    compartments: COMPARTMENTS,
};

pub const CONSTANTS_B: TissueConstants = TissueConstants {
    half_times_n2: &HALF_TIMES_N2,
    half_times_he: &HALF_TIMES_HE,
    he_a_s: &HE_AS,
    he_b_s: &HE_BS,
    n2_a_s: &N2AS_B,
    n2_b_s: &N2BS,
    compartments: COMPARTMENTS,
};

pub const CONSTANTS_C: TissueConstants = TissueConstants {
    half_times_n2: &HALF_TIMES_N2,
    half_times_he: &HALF_TIMES_HE,
    he_a_s: &HE_AS,
    he_b_s: &HE_BS,
    n2_a_s: &N2AS_C,
    n2_b_s: &N2BS,
    compartments: COMPARTMENTS,
};

pub struct TissueConstants<'a> {
    pub half_times_n2: &'a [f64], // = halfTimesN2;
    pub half_times_he: &'a [f64], // = halfTimesHe;
    pub he_a_s: &'a [f64],
    pub he_b_s: &'a [f64],
    pub n2_a_s: &'a [f64],
    pub n2_b_s: &'a [f64],
    pub compartments: usize,
}

/*

   Fresh Water = 1000kg/m³
   EN13319 = 1020 kg/m³
   Salt Water = 1030 kg/m³

*/
/*
Use mm for distance (10ft = 3048mm)
Use mbar for pressure.
 */
#[derive(Copy, Clone)]
pub struct Dive {
    pub gf_lo: f64,
    pub gf_hi: f64,
    pub dive_type: DiveType,
    pub deco_setpoint: f64,
    pub ascent_rate: DepthChange,
    pub descent_rate: DepthChange,
    pub atm_pressure: Pressure,
    pub last_stop: Pressure,
    pub stop_size: Pressure,
    pub metric: bool,
    pub partial_water: f64,
}

impl Default for Dive {
    fn default() -> Dive {
        let atm = Pressure::millibar(1013.0);
        Dive {
            gf_lo: 0.5,
            gf_hi: 0.8,
            dive_type: DiveType::OC,
            deco_setpoint: 1.3,
            ascent_rate: DepthChange::ascent_depth(Depth::meters(10.0)),
            descent_rate: DepthChange::descent_depth(Depth::meters(18.0)),
            atm_pressure: atm,
            last_stop: Pressure::from_depth(Depth::meters(3.0), atm),
            stop_size: Pressure::from_depth_rel(Depth::meters(3.0)),
            metric: true,
            partial_water: PARTIAL_WATER,
        }
    }
}

fn find_gas_for_setpoint(dil: Gas, setpoint: f64, atm: Pressure) -> Gas {
    let o2percent = setpoint / atm.to_bar();
    if o2percent < dil.f_o2 {
        return dil;
    }
    let he_percent = (dil.f_he / (dil.f_n2 + dil.f_he)) * (1.0 - o2percent);
    Gas::new_bottom(o2percent, he_percent, setpoint)
}

fn find_ocgas(gasses: &[Gas], depth: Pressure, segment_type: SegmentType) -> Gas {
    let mut ret: Option<Gas> = None;
    for g in gasses {
        if g.use_gas(depth, segment_type) {
            match ret {
                Some(rgas) => {
                    if g.f_o2 > rgas.f_o2 {
                        ret = Some(*g);
                    }
                }
                None => ret = Some(*g),
            }
        }
    }
    match ret {
        None => {
            for g in gasses {
                if g.use_gas(depth, segment_type) {
                    match ret {
                        Some(rgas) => {
                            if g.f_o2 * depth.to_mbar() < rgas.f_o2 * depth.to_mbar() {
                                ret = Some(*g);
                            }
                        }
                        None => ret = Some(*g),
                    }
                }
            }
        }
        _ => {}
    }
    // Seem to have no gasses... Return good old air.
    match ret {
        Some(gas) => gas,
        None => Gas::new_bottom(0.21, 0.0, 1.4),
    }
}

pub fn find_gas(
    dive: &Dive,
    gasses: &[Gas],
    depth: Pressure,
    segment_type: SegmentType,
    setpoint: f64,
) -> Gas {
    match dive.dive_type {
        DiveType::CCR => {
            let mut dil = Gas::new_bottom(0.21, 0.0, 1.4);
            for g in gasses {
                if g.use_diluent {
                    dil = *g;
                }
            }
            find_gas_for_setpoint(dil, setpoint, depth)
        }
        DiveType::OC => find_ocgas(gasses, depth, segment_type),
    }
}

fn next_gf(gf_slope: f64, dive: &Dive, stop: Pressure) -> f64 {
    if stop.to_mbar() - dive.stop_size.to_mbar() - dive.atm_pressure.to_mbar() < 0.0 {
        dive.gf_hi
    } else {
        (gf_slope * (stop.to_mbar() - dive.stop_size.to_mbar() - dive.atm_pressure.to_mbar()))
            + dive.gf_hi
    }
}

fn calc_ceiling(
    comps: &Compartments,
    atm: Pressure,
    constants: &TissueConstants,
    gf: f64,
) -> Pressure // Depth of current ceiling.
{
    let mut ceiling = 0.0;
    for i in 0..constants.compartments {
        let a = ((constants.n2_a_s[i] * comps.nitrogen[i])
            + (constants.he_a_s[i] * comps.helium[i]))
            / (comps.nitrogen[i] + comps.helium[i]);
        let b = ((constants.n2_b_s[i] * comps.nitrogen[i])
            + (constants.he_b_s[i] * comps.helium[i]))
            / (comps.nitrogen[i] + comps.helium[i]);
        let ceil = ((comps.nitrogen[i] + comps.helium[i]) - (gf * a)) / ((gf / b) - gf + 1.0);
        if ceil > ceiling {
            ceiling = ceil
        };
    }
    let stop = Pressure::bar(ceiling);
    if stop < atm {
        atm
    } else {
        stop
    }
}

fn calc_tissue_bottom(
    tissue_in: f64,
    time: f64,
    depth: Pressure,
    partial_water: f64,
    partial_pressure: f64,
    half_time: f64,
) -> f64 {
    let bar = depth.to_bar();
    let po = tissue_in;
    let pio = (bar - (partial_water / 1000.0)) * partial_pressure;
    po + (pio - po) * (1.0 - (2.0 as f64).powf(-time / half_time))
}

fn calc_bottom(
    comps_in: &Compartments,
    constants: &TissueConstants,
    partial_water: f64,
    depth: Pressure,
    time: f64,
    gas: Gas,
) -> Compartments {
    let mut comps_out = Compartments::new_empty(COMPARTMENTS);
    if time > 0.0 {
        for i in 0..constants.compartments {
            comps_out.nitrogen[i] = calc_tissue_bottom(
                comps_in.nitrogen[i],
                time,
                depth,
                partial_water,
                gas.f_n2,
                constants.half_times_n2[i],
            );
            comps_out.helium[i] = calc_tissue_bottom(
                comps_in.helium[i],
                time,
                depth,
                partial_water,
                gas.f_he,
                constants.half_times_he[i],
            );
        }
    }
    comps_out
}

fn calc_tissue_change(
    tissue_in: f64,
    time: f64,
    depth: Pressure,
    rate: DepthChange,
    partial_water: f64,
    partial_pressure: f64,
    half_time: f64,
) -> f64 {
    let bar = depth.to_bar();
    let rate_bar = rate.to_bar(); // rate of decent in bar
    let po = tissue_in;
    let pio = (bar - (partial_water / 1000.0)) * partial_pressure;
    let r = rate_bar * partial_pressure;
    let k = (2.0 as f64).ln() / half_time;
    pio + r * (time - (1.0 / k)) - (pio - po - (r / k)) * (-k * time).exp()
}

// rate_mbar should be negative on ascent
fn calc_change(
    comps_in: &Compartments,
    constants: &TissueConstants,
    partial_water: f64,
    gas: Gas,
    rate: DepthChange,
    from_depth: Pressure,
    to_depth: Pressure,
) -> Compartments {
    let mut comps_out = Compartments::new_copy(comps_in);
    let time: f64 = (to_depth.to_mbar() - from_depth.to_mbar()) / rate.to_mbar();
    for i in 0..constants.compartments {
        comps_out.nitrogen[i] = calc_tissue_change(
            comps_in.nitrogen[i],
            time,
            from_depth,
            rate,
            partial_water,
            gas.f_n2,
            constants.half_times_n2[i],
        );
        comps_out.helium[i] = calc_tissue_change(
            comps_in.helium[i],
            time,
            from_depth,
            rate,
            partial_water,
            gas.f_he,
            constants.half_times_he[i],
        );
    }
    comps_out
}

fn next_stop(dive: &Dive, comps: &Compartments, constants: &TissueConstants, gf: f64) -> Pressure // Depth of next stop.
{
    let stop = calc_ceiling(comps, dive.atm_pressure, constants, gf);
    if stop <= dive.atm_pressure {
        return dive.atm_pressure;
    }
    if stop.to_mbar() <= dive.last_stop.to_mbar() {
        return dive.last_stop;
    }
    let mut i = dive.last_stop.to_mbar() + dive.stop_size.to_mbar();
    while stop.to_mbar() > i {
        i += dive.stop_size.to_mbar();
    }
    Pressure::millibar(i as f64)
}

fn descend(
    dive: &Dive,
    constants: &TissueConstants,
    gasses: &[Gas],
    rate: DepthChange,
    from_depth: Pressure,
    to_depth: Pressure,
    setpoint: f64,
    comps_in: &Compartments,
) -> (Compartments, Segment) {
    let segment_type = if rate.to_mbar() < 0.0 {
        SegmentType::UP
    } else {
        SegmentType::DOWN
    };
    let time: f64 = (to_depth.to_mbar() - from_depth.to_mbar()) / rate.to_mbar();
    let gas: Gas = find_gas(dive, gasses, to_depth, segment_type, setpoint);
    let comps_out = calc_change(
        comps_in,
        constants,
        dive.partial_water,
        gas,
        rate,
        from_depth,
        to_depth,
    );
    let otu_cns = ::otu_cns::descent(rate, from_depth, to_depth, gas);

    let segment = Segment {
        segment_type,
        depth: to_depth,
        raw_time: time,
        time: time.ceil() as u32,
        gas,
        ceiling: 0,
        otu_cns,
        setpoint,
        compartments: Compartments::new_copy(&comps_out),
    };
    (comps_out, segment)
}

fn merge_ascends(prev_seg: Option<Segment>, new_seg: Segment) -> Vec<Segment> {
    let mut segs: Vec<Segment> = Vec::new();
    let mut time = new_seg.raw_time;
    let mut otu_cns = new_seg.otu_cns;
    match prev_seg {
        Some(seg) => {
            if seg.segment_type == SegmentType::UP && seg.gas == new_seg.gas {
                time += seg.raw_time;
                otu_cns += seg.otu_cns;
                let seg_out = Segment {
                    time: time.ceil() as u32,
                    raw_time: time,
                    otu_cns,
                    compartments: Compartments::new_copy(&new_seg.compartments),
                    ..new_seg
                };
                segs.push(seg_out);
                return segs;
            } else {
                segs.push(seg);
            }
        }
        None => {}
    };
    segs.push(new_seg);
    segs
}

fn ascend(
    dive: &Dive,
    constants: &TissueConstants,
    gasses: &[Gas],
    rate: DepthChange,
    from_depth: Pressure,
    to_depth: Pressure,
    setpoint: f64,
    comps_in: &Compartments,
) -> (Compartments, Segment) {
    descend(
        dive, constants, gasses, rate, from_depth, to_depth, setpoint, comps_in,
    )
}

fn bottom(
    dive: &Dive,
    constants: &TissueConstants,
    gasses: &[Gas],
    depth: Pressure,
    time: f64,
    setpoint: f64,
    comps_in: &Compartments,
) -> (Compartments, Segment) {
    let gas = find_gas(dive, gasses, depth, SegmentType::LEVEL, setpoint);
    let comps_out = calc_bottom(comps_in, constants, dive.partial_water, depth, time, gas);
    let ceiling = calc_ceiling(comps_in, dive.atm_pressure, constants, dive.gf_lo);
    let otu_cns = ::otu_cns::bottom(depth, time, gas);
    let new_comps = Compartments::new_copy(&comps_out);
    (
        comps_out,
        Segment {
            segment_type: SegmentType::LEVEL,
            depth,
            raw_time: time,
            time: time.ceil() as u32,
            gas,
            ceiling: ceiling.to_mbar() as i32,
            otu_cns,
            setpoint,
            compartments: new_comps,
        },
    )
}

fn calc_bottom_segment(
    dive: &Dive,
    comps_in: &Compartments,
    constants: &TissueConstants,
    gas: Gas,
    depth: Pressure,
    gf: f64,
    time_in: f64,
) -> (Compartments, f64) {
    let mut comps_out = Compartments::new_copy(comps_in);
    let mut time = 0.0;
    let mut done = false;
    let mut first = true;
    while !done {
        let segment_time = if first { 1.0 - time_in } else { 1.0 };
        comps_out = calc_bottom(
            &comps_out,
            constants,
            dive.partial_water,
            depth,
            segment_time,
            gas,
        );
        time += 1.0;
        let nfs = next_stop(dive, &comps_out, constants, gf);
        done = nfs < depth;
        first = false;
    }
    (comps_out, time)
}

fn calc_deco_int(
    dive: &Dive,
    comps_in: &Compartments,
    constants: &TissueConstants,
    gasses: &[Gas],
    last_depth_in: Pressure,
    gf: f64,
    gf_slope: f64,
) -> (Vec<Segment>, Compartments) {
    let mut segments: Vec<Segment> = Vec::new();
    let mut main_done = false;
    let mut ngf = gf;
    let mut last_depth = last_depth_in;
    let mut nfs: Pressure;
    let mut comps_out = Compartments::new_copy(comps_in);
    while !main_done {
        let fs = next_stop(dive, &comps_out, constants, ngf);
        if fs < last_depth {
            let (comps, seg) = ascend(
                dive,
                constants,
                gasses,
                dive.ascent_rate,
                last_depth,
                fs,
                dive.deco_setpoint,
                &comps_out,
            );
            let mut newsegs = merge_ascends(segments.pop(), seg);
            segments.append(&mut newsegs);
            comps_out = comps;
            last_depth = fs;
            let seg = &segments[segments.len() - 1];
            if seg.raw_time > 1.0 && seg.time as f64 > seg.raw_time {
                let time_off = seg.time as f64 - seg.raw_time;
                comps_out = calc_bottom(
                    &comps_out,
                    constants,
                    dive.partial_water,
                    fs,
                    time_off,
                    seg.gas,
                );
            }
        }
        if fs <= dive.atm_pressure {
            return (segments, comps_out);
        } // At surface, done...
        ngf = next_gf(gf_slope, dive, fs);
        nfs = next_stop(dive, &comps_out, constants, ngf);
        if nfs == fs {
            let gas = find_gas(dive, gasses, fs, SegmentType::UP, dive.deco_setpoint);
            // XXX I want to be a function...
            let time_off = if segments.is_empty() {
                0.0
            } else {
                if segments[segments.len() - 1].segment_type != SegmentType::LEVEL
                    && segments[segments.len() - 1].raw_time < 1.0
                {
                    segments[segments.len() - 1].raw_time
                } else {
                    0.0
                }
            };
            if time_off > 0.0 {
                segments.pop();
            } // What about otu/cns? XXX TODO
            let (new_comps, time) =
                calc_bottom_segment(dive, &comps_out, constants, gas, fs, ngf, time_off);
            comps_out = new_comps;
            nfs = next_stop(dive, &comps_out, constants, ngf);
            let otu_cns = ::otu_cns::bottom(fs, time, gas);
            last_depth = fs;
            segments.push(Segment {
                segment_type: SegmentType::LEVEL,
                depth: fs,
                raw_time: time,
                time: time.ceil() as u32,
                gas,
                ceiling: 0,
                otu_cns,
                setpoint: dive.deco_setpoint,
                compartments: Compartments::new_copy(&comps_out),
            });
        }
        main_done = nfs <= dive.atm_pressure;
    }
    (segments, comps_out)
}

fn initial_segments(
    dive: &Dive,
    compartments: &Compartments,
    constants: &TissueConstants,
    segments_in: &[SegmentIn],
    gasses: &[Gas],
) -> (Vec<Segment>, Compartments, Pressure) {
    let mut comps_out = Compartments::new_copy(compartments);
    let mut segments: Vec<Segment> = Vec::new();
    let mut last_depth = dive.atm_pressure;
    for s in segments_in {
        let depth = Pressure::from_depth(s.depth, dive.atm_pressure);
        let raw_time: f64;
        if last_depth < depth {
            let (comps, seg) = descend(
                dive,
                constants,
                gasses,
                dive.descent_rate,
                last_depth,
                depth,
                s.setpoint,
                &comps_out,
            );
            raw_time = seg.raw_time;
            segments.push(seg);
            comps_out = comps;
        } else {
            let (comps, seg) = ascend(
                dive,
                constants,
                gasses,
                dive.ascent_rate,
                last_depth,
                depth,
                s.setpoint,
                &comps_out,
            );
            raw_time = seg.raw_time;
            segments.push(seg);
            comps_out = comps;
        }
        let (comps, seg) = bottom(
            dive,
            constants,
            gasses,
            depth,
            s.time - raw_time,
            s.setpoint,
            &comps_out,
        );
        comps_out = comps;
        segments.push(Segment {
            time: (s.time - raw_time.ceil()) as u32,
            ..seg
        });
        last_depth = depth;
    }
    (segments, comps_out, last_depth)
}

fn ascend_to_first_stop(
    dive: &Dive,
    compartments: &Compartments,
    constants: &TissueConstants,
    gasses: &[Gas],
    depth: Pressure,
) -> (Vec<Segment>, Compartments, Pressure) {
    let mut last_depth = depth;
    let mut segments: Vec<Segment> = Vec::new();
    let mut comps_out = Compartments::new_copy(compartments);
    let mut fs = next_stop(dive, &comps_out, constants, dive.gf_lo);
    let mut at_first_stop = false;
    while !at_first_stop {
        let (comps, seg) = ascend(
            dive,
            constants,
            gasses,
            dive.ascent_rate,
            last_depth,
            fs,
            dive.deco_setpoint,
            &comps_out,
        );
        comps_out = comps;
        let mut newsegs = merge_ascends(segments.pop(), seg);
        segments.append(&mut newsegs);
        //comps_out = comps;

        // Comment next couple lines (exit loop) out to start gf slope at natural first stop even
        // if it has cleared in the ascent to it- leaving them in seems to match
        // Shearwater closer and not Subsurface...
        last_depth = fs;
        fs = next_stop(dive, &comps_out, constants, dive.gf_lo);
        at_first_stop = fs >= last_depth;
    }

    last_depth = fs;
    (segments, comps_out, last_depth)
}

pub fn calc_deco(
    dive: &Dive,
    compartments: &Compartments,
    constants: &TissueConstants,
    segments_in: &[SegmentIn],
    gasses: &[Gas],
) -> Result<Vec<Segment>, String> {
    if segments_in.len() == 0 {
        return Err("Must provide segment(s) to calculate deco against.".to_string());
    }
    let (mut segments, comps_out, last_depth) =
        initial_segments(dive, compartments, constants, segments_in, gasses);
    // Ascend to the first stop.
    let (mut newsegs, comps_out, last_depth) =
        ascend_to_first_stop(dive, &comps_out, constants, gasses, last_depth);
    segments.append(&mut newsegs);
    let gf_slope =
        (dive.gf_hi - dive.gf_lo) / -((last_depth.to_mbar() - dive.atm_pressure.to_mbar()) as f64);
    let gf = next_gf(gf_slope, dive, last_depth);
    let (mut newsegs, _comps_out) = calc_deco_int(
        dive, &comps_out, constants, gasses, last_depth, gf, gf_slope,
    );
    segments.append(&mut newsegs);
    Ok(segments)
}

pub fn calc_deco_a(
    dive: &Dive,
    compartments: &Compartments,
    segments: &[SegmentIn],
    gasses: &[Gas],
) -> Result<Vec<Segment>, String> {
    calc_deco(dive, compartments, &CONSTANTS_A, segments, gasses)
}
pub fn calc_deco_b(
    dive: &Dive,
    compartments: &Compartments,
    segments: &[SegmentIn],
    gasses: &[Gas],
) -> Result<Vec<Segment>, String> {
    calc_deco(dive, compartments, &CONSTANTS_B, segments, gasses)
}
pub fn calc_deco_c(
    dive: &Dive,
    compartments: &Compartments,
    segments: &[SegmentIn],
    gasses: &[Gas],
) -> Result<Vec<Segment>, String> {
    calc_deco(dive, compartments, &CONSTANTS_C, segments, gasses)
}
