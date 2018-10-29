pub const COMPARTMENTS: usize = 16;

pub const N2AS_A: [f64; COMPARTMENTS] = [
    1.2599,
    1.0000,
    0.8618,
    0.7562,
    0.6667,
    0.5933,
    0.5282,
    0.4701,
    0.4187,
    0.3798,
    0.3497,
    0.3223,
    0.2971,
    0.2737,
    0.2523,
    0.2327
];
pub const N2AS_B: [f64; COMPARTMENTS] = [
    1.2599,
    1.0000,
    0.8618,
    0.7562,
    0.6667,
    0.5600,
    0.4947,
    0.4500,
    0.4187,
    0.3798,
    0.3497,
    0.3223,
    0.2850,
    0.2737,
    0.2523,
    0.2327
];
pub const N2AS_C: [f64; COMPARTMENTS] = [
    1.2599,
    1.0000,
    0.8618,
    0.7562,
    0.6200,
    0.5043,
    0.4410,
    0.4000,
    0.3750,
    0.3500,
    0.3295,
    0.3065,
    0.2835,
    0.2610,
    0.2480,
    0.2327
];


//pub const PARTIAL_WATER: f64 = 056.7;
pub const PARTIAL_WATER: f64 = 062.7;
//pub const PARTIAL_WATER: f64 = 049.3;

pub const HALF_TIMES_N2: [f64; COMPARTMENTS] = [
    4.00,
    8.00,
    12.50,
    18.50,
    27.00,
    38.30,
    54.30,
    77.00,
    109.00,
    146.00,
    187.00,
    239.00,
    305.00,
    390.00,
    498.00,
    635.00
]; // 1b 5.0
pub const HALF_TIMES_HE: [f64; COMPARTMENTS] = [
    1.51,
    3.02,
    4.72,
    6.99,
    10.21,
    14.48,
    20.53,
    29.11,
    41.20,
    55.19,
    70.69,
    90.34,
    115.29,
    147.42,
    188.24,
    240.03
]; // 1b 1.88
pub const HE_AS: [f64; COMPARTMENTS] = [
    1.7424,
    1.3830,
    1.1919,
    1.0458,
    0.9220,
    0.8205,
    0.7305,
    0.6502,
    0.5950,
    0.5545,
    0.5333,
    0.5189,
    0.5181,
    0.5176,
    0.5172,
    0.5119
];
pub const HE_BS: [f64; COMPARTMENTS] = [
    0.4245,
    0.5747,
    0.6527,
    0.7223,
    0.7582,
    0.7957,
    0.8279,
    0.8553,
    0.8757,
    0.8903,
    0.8997,
    0.9073,
    0.9122,
    0.9171,
    0.9217,
    0.9267
];
pub const N2BS: [f64; COMPARTMENTS] = [
    0.5050,
    0.6514,
    0.7222,
    0.7825,
    0.8126,
    0.8434,
    0.8693,
    0.8910,
    0.9092,
    0.9222,
    0.9319,
    0.9403,
    0.9477,
    0.9544,
    0.9602,
    0.9653
];
