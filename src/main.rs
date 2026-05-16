use std::process::exit;

mod controller;
mod cpp_templates;
mod model;
mod ui;
mod utils;
mod wrapper;

fn main() {
    exit(controller::execute());
}