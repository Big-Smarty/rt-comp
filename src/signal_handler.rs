#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

use log::*;

pub struct SignalHandler {}

impl SignalHandler {
    pub fn new() -> SignalHandler {
        SignalHandler {}
    }
}
