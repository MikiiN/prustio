use std::path::PathBuf;

pub fn ensure_dir_existence(path: &PathBuf) -> Result<(), String> {
    if !path.exists() {
        match std::fs::create_dir(path) {
            Ok(_) => (),
            Err(_) => {
                let name = match path.as_os_str().to_str() {
                    Some(n) => n,
                    None => "invalid path",
                };
                let error_msg = format!("Can not create directory:\n{name}");
                return Err(error_msg);
            },
        }
    }
    Ok(())
}