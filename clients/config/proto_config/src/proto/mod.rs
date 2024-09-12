#![allow(warnings)]

pub use self::config::*;

include!(concat!(env!("OUT_DIR"), "/src/proto/gen.rs"));
