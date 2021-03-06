extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate rand;
extern crate time;

extern crate rayon;

extern crate arrayvec;

#[macro_use]
#[cfg(feature = "heuristic-random")]
extern crate lazy_static;

pub mod input;
pub mod engine;
pub mod strategy;
