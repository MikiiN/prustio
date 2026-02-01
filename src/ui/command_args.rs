pub enum CommandArgs {
    ProjInit(ProjInitArguments),
    ProjAdd(ProjAddRemoveArguments),
    ProjRemove(ProjAddRemoveArguments),
    ProjTasks(ProjTasksArguments),
    Run(RunArguments),
}

//
//  Project commands
//

pub struct ProjInitArguments {
    name: Option<String>,
    board: Option<String>,
    hybrid: bool,
    json_output: bool,
}

impl ProjInitArguments {
    pub fn new(
        name: &Option<String>, 
        board: &Option<String>, 
        hybrid: &bool, 
        json_output: &bool
    ) -> Self {
        ProjInitArguments { 
            name: name.clone(), 
            board: board.clone(), 
            hybrid: hybrid.clone(), 
            json_output: json_output.clone(), 
        }
    }
}

pub struct ProjAddRemoveArguments {
    package: Option<String>, 
    json_output: bool,
}

impl ProjAddRemoveArguments {
    pub fn new(package: &Option<String>, json_output: &bool) -> Self {
        ProjAddRemoveArguments { 
            package: package.clone(), 
            json_output: json_output.clone(), 
        }
    }
}

pub struct ProjTasksArguments {
    json_output: bool,
}

impl ProjTasksArguments {
    pub fn new(json_output: &bool) -> Self {
        ProjTasksArguments { json_output: json_output.clone() }
    }
}

//
//  Run commands
//

pub struct RunArguments {
    target: Option<String>,
    json_output: bool,
}

impl RunArguments {
    pub fn new(target: &Option<String>, json_output: &bool) -> Self {
        RunArguments { 
            target: target.clone(), 
            json_output: json_output.clone(), 
        }
    }
}