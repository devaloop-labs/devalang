pub mod arp;
pub mod pad;
pub mod pluck;
pub mod sub;

use devalang_types::Value;
use std::collections::HashMap;

pub fn apply_type(synth_params: &mut HashMap<String, Value>) {
    if let Some(tval) = synth_params.get("type") {
        let tname = match tval {
            Value::String(s) => s.as_str(),
            Value::Identifier(s) => s.as_str(),
            _ => "",
        };
        match tname {
            "pad" => pad::apply_defaults(synth_params),
            "pluck" => pluck::apply_defaults(synth_params),
            "arp" => arp::apply_defaults(synth_params),
            "sub" => sub::apply_defaults(synth_params),
            _ => {}
        }
    }
}
