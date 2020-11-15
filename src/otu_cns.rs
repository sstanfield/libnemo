use crate::gas::Gas;
use crate::types::*;

use std::ops::Add;
use std::ops::AddAssign;

const _CNS_PPO2SEGMENTS: usize = 7;
const _CNS_PPO2LO: [f64; 7] = [0.5, 0.6, 0.7, 0.8, 0.9, 1.1, 1.5];
const _CNS_PPO2HI: [f64; 7] = [0.6, 0.7, 0.8, 0.9, 1.1, 1.5, 100.0];
const _CNS_LIMIT_SLOPE: [f64; 7] = [-1800.0, -1500.0, -1200.0, -900.0, -600.0, -300.0, -750.0];
const _CNS_LIMIT_INTERCEPT: [f64; 7] = [1800.0, 1620.0, 1410.0, 1170.0, 900.0, 570.0, 1245.0];

#[derive(Copy, Clone)]
pub struct OtuCns {
    pub otu: f64,
    pub cns: f64,
}

impl Add for OtuCns {
    type Output = OtuCns;

    fn add(self, other: OtuCns) -> OtuCns {
        OtuCns {
            otu: self.otu + other.otu,
            cns: self.cns + other.cns,
        }
    }
}

impl AddAssign for OtuCns {
    fn add_assign(&mut self, other: OtuCns) {
        *self = OtuCns {
            otu: self.otu + other.otu,
            cns: self.cns + other.cns,
        };
    }
}

/// Algorithm initially from paper:
/// Oxygen Toxicity Calculations by Erik C. Baker, P.E.
/// Link as of writing at: https://www.shearwater.com/wp-content/uploads/2012/08/Oxygen_Toxicity_Calculations.pdf
/// Calculates otu and cns for a bottom segment.
pub fn bottom(depth: Pressure, time: f64, gas: Gas) -> OtuCns {
    let po2 = gas.f_o2 * (depth.to_mbar() / 1000.0);
    let otu = if po2 <= 0.5 {
        0.0
    } else {
        time * ((0.5 / (po2 - 0.5)).powf(-5.0 / 6.0))
    };
    let mut cns = 0.0;
    let mut tlim = 0.0;
    if po2 > _CNS_PPO2LO[0] {
        for x in 0.._CNS_PPO2SEGMENTS {
            if po2 > _CNS_PPO2LO[x] && po2 <= _CNS_PPO2HI[x] {
                tlim = _CNS_LIMIT_SLOPE[x] * po2 + _CNS_LIMIT_INTERCEPT[x];
            }
        }
        cns = if tlim > 0.0 { time / tlim } else { 0.0 };
    }
    OtuCns {
        otu,
        cns: cns * 100.0,
    }
}

/// Algorithm initially from paper:
/// Oxygen Toxicity Calculations by Erik C. Baker, P.E.
/// Link as of writing at: https://www.shearwater.com/wp-content/uploads/2012/08/Oxygen_Toxicity_Calculations.pdf
/// Calculates otu and cns for an ascent/descent segment.
pub fn descent(
    rate_mbar: DepthChange,
    from_depth: Pressure,
    to_depth: Pressure,
    gas: Gas,
) -> OtuCns {
    let time = (to_depth.to_mbar() - from_depth.to_mbar()) / rate_mbar.to_mbar();
    let maxata = to_depth.to_mbar().max(from_depth.to_mbar()) / 1000.0;
    let minata = to_depth.to_mbar().min(from_depth.to_mbar()) / 1000.0;
    let maxpo2 = gas.f_o2 * maxata;
    let minpo2 = gas.f_o2 * minata;
    let mut otu = 0.0;
    let mut cns = 0.0;
    if maxpo2 > 0.5 {
        let lowpo2 = if minpo2 < 0.5 { 0.5 } else { minpo2 };
        let time = time * (maxpo2 - lowpo2) / (maxpo2 - minpo2);
        otu = 3.0 / 11.0 * time / (maxpo2 - lowpo2)
            * (((maxpo2 - 0.5f64) / 0.5f64).powf(11.0 / 6.0))
            - (((lowpo2 - 0.5f64) / 0.5f64).powf(11.0 / 6.0));
        let mut otime = Vec::with_capacity(_CNS_PPO2SEGMENTS);
        otime.resize(_CNS_PPO2SEGMENTS, 0.0);
        let mut po2o = Vec::with_capacity(_CNS_PPO2SEGMENTS);
        po2o.resize(_CNS_PPO2SEGMENTS, 0.0);
        let mut po2f = Vec::with_capacity(_CNS_PPO2SEGMENTS);
        po2f.resize(_CNS_PPO2SEGMENTS, 0.0);
        let mut segpo2 = Vec::with_capacity(_CNS_PPO2SEGMENTS);
        segpo2.resize(_CNS_PPO2SEGMENTS, 0.0);
        let up = from_depth > to_depth;
        for i in 0.._CNS_PPO2SEGMENTS {
            if maxpo2 > _CNS_PPO2LO[i] && lowpo2 <= _CNS_PPO2HI[i] {
                if (maxpo2 >= _CNS_PPO2HI[i]) && (lowpo2 < _CNS_PPO2LO[i]) {
                    po2o[i] = if up { _CNS_PPO2HI[i] } else { _CNS_PPO2LO[i] };
                    po2f[i] = if !up { _CNS_PPO2HI[i] } else { _CNS_PPO2LO[i] };
                } else if (maxpo2 < _CNS_PPO2HI[i]) && (lowpo2 <= _CNS_PPO2LO[i]) {
                    po2o[i] = if up { maxpo2 } else { _CNS_PPO2LO[i] };
                    po2f[i] = if !up { maxpo2 } else { _CNS_PPO2LO[i] };
                } else if (lowpo2 > _CNS_PPO2LO[i]) && (maxpo2 >= _CNS_PPO2HI[i]) {
                    po2o[i] = if up { _CNS_PPO2HI[i] } else { lowpo2 };
                    po2f[i] = if !up { _CNS_PPO2HI[i] } else { lowpo2 };
                } else {
                    po2o[i] = if up { maxpo2 } else { lowpo2 };
                    po2f[i] = if !up { maxpo2 } else { lowpo2 };
                }
                segpo2[i] = po2f[i] - po2o[i];
                otime[i] = time * (segpo2[i] / (maxpo2 - lowpo2)).abs();
            } else {
                otime[i] = 0.0;
                po2o[i] = 0.0;
                po2f[i] = 0.0;
                segpo2[i] = 0.0;
            }
        }
        for i in 0.._CNS_PPO2SEGMENTS {
            if otime[i] > 0.0 {
                let tlimi = _CNS_LIMIT_SLOPE[i] * po2o[i] + _CNS_LIMIT_INTERCEPT[i];
                let mk = _CNS_LIMIT_SLOPE[i] * (segpo2[i] / otime[i]);
                cns += 1.0 / mk * (((tlimi + mk * otime[i]).abs()).ln() - (tlimi.abs()).ln());
            }
        }
    }

    OtuCns {
        otu,
        cns: cns * 100.0,
    }
}
