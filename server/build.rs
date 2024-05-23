#[path = "src/main.rs"]
mod main;

use dropshot::ApiDescription;
use main::routes::counter::{get_counter, put_counter};
use std::fs::File;

fn main() {
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/counter.rs");
    println!("cargo:rerun-if-changed=src/context.rs");

    // write out the open api server spec
    let mut api = ApiDescription::new();
    api.register(get_counter).unwrap();
    api.register(put_counter).unwrap();
    let mut file = File::create("api/v1.json").unwrap();
    api.openapi("petstore", "").write(&mut file).unwrap();
}
