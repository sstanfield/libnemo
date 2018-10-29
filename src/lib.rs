pub mod types;
pub use types::*;

pub mod dive_consts;
pub use dive_consts::*;

pub mod segment_type;
pub use segment_type::*;

pub mod segment;
pub use segment::*;

pub mod gas;
pub use gas::*;

pub mod otu_cns;
pub use otu_cns::*;

pub mod dive;
pub use dive::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
