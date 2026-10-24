use crate::Result;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    env, fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

pub fn app_directory(variable: &str, fallback: &str) -> Result<PathBuf> {
    let base = env::var_os(variable)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(fallback)))
        .ok_or_else(|| format!("Neither {variable} nor HOME is available"))?;
    Ok(base.join("iot-display"))
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes =
        fs::read(path).map_err(|error| format!("Cannot read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("Invalid {}: {error}", path.display()))
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let directory = path.parent().ok_or("Storage path has no parent")?;
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    fs::set_permissions(directory, fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())?;
    // A rename in the same directory is atomic; partial writes never replace the old file.
    let mut file = tempfile::NamedTempFile::new_in(directory).map_err(|error| error.to_string())?;
    file.as_file()
        .set_permissions(fs::Permissions::from_mode(0o600))
        .map_err(|error| error.to_string())?;
    serde_json::to_writer_pretty(&mut file, value).map_err(|error| error.to_string())?;
    file.write_all(b"\n").map_err(|error| error.to_string())?;
    file.as_file()
        .sync_all()
        .map_err(|error| error.to_string())?;
    file.persist(path)
        .map_err(|error| error.error.to_string())?;
    fs::File::open(directory)
        .and_then(|file| file.sync_all())
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_atomically_with_private_permissions() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("app/config.json");
        write_json(&path, &vec![1, 2]).unwrap();
        write_json(&path, &vec![3]).unwrap();
        assert_eq!(read_json::<Vec<u32>>(&path).unwrap(), vec![3]);
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(path.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }

    #[test]
    fn failed_save_does_not_replace_existing_file() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("config.json");
        write_json(&path, &"original").unwrap();
        assert!(write_json(&path.join("child"), &"replacement").is_err());
        assert_eq!(read_json::<String>(&path).unwrap(), "original");
    }
}
