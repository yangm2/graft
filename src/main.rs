use clap::{crate_authors, crate_description, crate_name, crate_version, Arg, Command, ValueHint};
use std::error;
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

// Custom error type
enum CliError {
    DstDirNotEmpty(String),
    NotDir(PathBuf),
    WrongArgs,
}

impl error::Error for CliError {
    fn cause(&self) -> Option<&dyn error::Error> {
        Some(self)
    }
}

impl fmt::Debug for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
            CliError::DstDirNotEmpty(s) => write!(f, "Destination directory ({}) is not empty", s),
            CliError::NotDir(s) => write!(f, "Not a Directory: {}", s.display()),
            CliError::WrongArgs => write!(f, "Wrong number of args"),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

fn check_dir(p: &PathBuf) -> Result<PathBuf, CliError> {
    // TODO: is the dir Readable?  eXecutable?
    match Path::new(p).is_dir() {
        true => Ok(p.to_path_buf()),
        false => Err(CliError::NotDir(p.to_path_buf())),
    }
}

fn parse_arg_with_clap() -> Result<PathBuf, CliError> {
    let app = Command::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(
            Arg::new("DIR")
                .help("source directory path")
                .value_hint(ValueHint::DirPath)
                .value_parser(clap::builder::PathBufValueParser::new())
                .required(true)
                .index(1),
        )
        .get_matches();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(i) = app.get_one::<PathBuf>("DIR") {
        println!("Value for DIR: {}", i.display());
    }

    app.get_one::<PathBuf>("DIR")
        .ok_or(CliError::WrongArgs)
        .and_then(check_dir)
}

// Algorithm:
//   1. create subdirectory tree
//   2. if not symlink, create symlink
//   3. if symlink, copy symlink (both absolute and relative)
fn recurse(srcdir: &Path, cwd: &Path) -> std::io::Result<()> {
    use std::fs;
    use std::os::unix::fs as unix_fs;

    for wrapped_entry in srcdir.read_dir()? {
        let entry = wrapped_entry?;

        let entry_path = entry.path();
        let ft_data = entry.metadata()?.file_type();

        let target = cwd.canonicalize()?.join(entry.file_name());

        if ft_data.is_dir() {
            // create local subdir ...
            fs::create_dir(&target)?;

            // then decend
            recurse(&entry_path, &target)?;
        } else if !ft_data.is_symlink() {
            // create symlink
            unix_fs::symlink(&entry_path, &target)?;
        } else if ft_data.is_symlink() {
            // copy symlink
            unix_fs::symlink(&fs::read_link(entry_path)?, &target)?;
        }
    }

    // TODO: error handling?!?!?!
    Ok(())
}

fn dir_is_empty(dst: &Path) -> Result<bool, CliError> {
    match dst.read_dir() {
        Ok(entry) => {
            if entry.count() > 0 {
                Ok(false)
            } else {
                Ok(true)
            }
        }
        Err(_e) => Err(CliError::NotDir(dst.to_path_buf())),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    match parse_arg_with_clap() {
        Ok(srcdir) => {
            // validate the destination directory
            let dstdir = Path::new(".");
            // error unless current dir (i.e. target) is empty
            if let Ok(empty) = dir_is_empty(dstdir) {
                if !empty {
                    return Err(Box::from(CliError::DstDirNotEmpty(
                        dstdir.canonicalize()?.display().to_string(),
                    )));
                }
            }

            let dir = srcdir.as_path();
            recurse(dir, dstdir).or_else(|_| Err(Box::from(CliError::NotDir(dir.canonicalize()?))))
        }
        Err(e) => Err(Box::from(e)),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[allow(non_snake_case)]
    fn test_check_dir_NotDir() {
        use super::*;

        assert_eq!(
            check_dir(&PathBuf::from(r"asdfasd"))
                .unwrap_err()
                .to_string(),
            String::from("Not a Directory: asdfasd")
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_check_dir_IsDir() {
        use super::*;

        assert_eq!(
            check_dir(&PathBuf::from(r"/dev"))
                .unwrap()
                .to_str()
                .unwrap(),
            String::from("/dev")
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_dir_is_empty_NotDir() {
        use super::*;

        assert_eq!(
            dir_is_empty(&Path::new("asdfasd")).unwrap_err().to_string(),
            String::from("Not a Directory: asdfasd")
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_dir_is_empty_False() {
        use super::*;

        // on Linux systems, "/" should always be a directory and never be empty
        assert_eq!(dir_is_empty(&Path::new("/")).unwrap(), false,);
    }
}
