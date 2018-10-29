extern crate libnemo;

use libnemo::*;

fn main() {
    let gasses = vec![Gas::new_bottom(0.18, 0.45, 1.4), Gas::new_deco(0.5, 0.0), Gas::new_deco(0.99, 0.0)];
    let segments = vec![SegmentIn::new_bottom(Depth::meters(60.0), 30.0, 1.4)];

    let dive = Dive {..Default::default()};
    let segs = calc_deco_c(&dive,
                         &Compartments::new_surface(1013.0, PARTIAL_WATER, COMPARTMENTS),
        &segments, &gasses
    ).unwrap();

    let mut tot = 0;
    let mut cns = 0.0;
    let mut otu = 0.0;
    for s in segs {
        println!("{}   {}({})   {} otu {} cns {}", s.depth.to_depth(dive.atm_pressure).to_meters(), s.time, s.raw_time, s.gas, s.otu_cns.otu, s.otu_cns.cns);
        tot += s.time;
        cns += s.otu_cns.cns;
        otu += s.otu_cns.otu;
    }
    println!("Run time: {}, cns: {}, otu: {}", tot, cns, otu);
}
