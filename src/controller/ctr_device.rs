
use crate::model::device;
use crate::ui::display;

pub fn device_list(json_output: &bool) {
    let devices = match device::get_port_list() {
        Ok(list) => list,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };
    if *json_output {
        display::print_devices_json(&devices);
    } else {
        display::print_devices_table(&devices);
    }
}