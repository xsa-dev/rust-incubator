use std::collections::BTreeMap;

use step_3_2::{btreemap, proc_btreemap};

fn main() {
    let declarative: BTreeMap<_, _> = btreemap! {
        "rust" => "awesome",
        "macros" => "powerful",
    };

    let procedural: BTreeMap<_, _> = proc_btreemap! {
        "rust" => "fearless",
        "macros" => "expressive",
    };

    println!("Declarative macro output: {declarative:?}");
    println!("Procedural macro output: {procedural:?}");
}
