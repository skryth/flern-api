use std::{fs::File, io::Read, path::PathBuf};

use tracing::debug;

use super::error::ConfigResult;

pub fn find_config_file(use_local: bool) -> PathBuf {
    let app_name = crate::APPLICATION_NAME;

    if use_local {
        return PathBuf::from("./config.toml");
    }

    #[cfg(unix)]
    let path = std::env::var_os("HOME");
    #[cfg(windows)]
    let path = std::env::var_os("APPDATA");

    #[cfg(any(unix, windows))]
    if let Some(app_path) = path {
        let mut path = PathBuf::from(app_path);

        if cfg!(unix) {
            path = path.join(".config");
        }

        path = path.join(app_name).join("config.toml");

        if path.exists() {
            return path;
        }
    }

    PathBuf::from("./config.toml")
}

pub fn read_config(use_local: bool) -> ConfigResult<Vec<u8>> {
    let filename = find_config_file(use_local);

    tracing::trace!("looking for config at: {}", filename.display());
    if !filename.exists() {
        return Err(crate::config::error::ConfigError::ConfigNotFound);
    }

    let filename = filename.canonicalize().expect("Unable to canonicalize config filename");
    debug!("using {} as configuration file", filename.display());

    let mut fd = File::open(filename)?;
    let mut buf = Vec::new();
    fd.read_to_end(&mut buf)?;

    Ok(buf)
}

#[cfg(test)]
mod test {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_find_config_file_local() {
        let path = find_config_file(true);
        assert_eq!(path, PathBuf::from("./config.toml"));
    }

    #[test]
    fn test_find_config_file_unix_home() {
        let temp_dir = tempfile::tempdir().unwrap();
        let fake_config = temp_dir
            .path()
            .join(".config")
            .join(crate::APPLICATION_NAME);
        fs::create_dir_all(&fake_config).unwrap();
        let config_file = fake_config.join("config.toml");
        fs::write(&config_file, "dummy = true").unwrap();

        #[cfg(unix)]
        unsafe {
            env::set_var("HOME", temp_dir.path());
        }

        #[cfg(windows)]
        unsafe {
            env::set_var("APPDATA", temp_dir.path());
        }

        let path = find_config_file(false);
        assert_eq!(path, config_file);
    }

    #[test]
    fn test_read_config_success() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("config.toml");
        fs::write(&file_path, b"foo = 'bar'").unwrap();

        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        let result = read_config(true);

        env::set_current_dir(original_dir).unwrap();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"foo = 'bar'");
    }
}
