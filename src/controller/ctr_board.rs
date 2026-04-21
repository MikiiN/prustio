use crate::model::board;
use crate::ui::display;

pub fn board(
    filter: Option<&String>,
    json_output: &bool,
) -> Result<(), String> {
    let boards = board::get_boards(filter);

    if *json_output {
        display::print_boards_json(&boards);
    } else {
        display::print_boards_table(&boards);
    }
    Ok(())
}