use std::path::PathBuf;

pub fn get_uploads_dir() -> std::io::Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    Ok(cwd.join("uploads"))
}
