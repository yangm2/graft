use clap::{App, Arg};
use std::error;
use std::error::Error;
use std::fmt;
use std::path::Path;

// Custom error type
enum CliError {
    DstDirNotEmpty(String),
    NotDir(String),
    WrongArgs,
}

impl error::Error for CliError {
    fn cause(&self) -> Option<&dyn error::Error> {
        Some(self)
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}

impl fmt::Debug for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
            CliError::DstDirNotEmpty(s) => write!(f, "Destination directory ({}) is not empty", s),
            CliError::NotDir(s) => write!(f, "Not a Directory: {}", s),
            CliError::WrongArgs => write!(f, "Wrong number of args"),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    fn parse_arg_with_clap() -> Result<String, CliError> {
        fn check(p: &str) -> Result<String, CliError> {
            // TODO: is the dir Readable?  eXecutable?
            match Path::new(&p).is_dir() {
                true => Ok(p.to_string()),
                false => Err(CliError::NotDir(p.to_string())),
            }
        }

        let matches = App::new("graft")
            .version("0.2")
            .author("Mike")
            .about("Replicate the structure of an original directory-tree and populate the replica with symlinks to the original directory-tree.")
            .arg(Arg::new("DIR")
                .about("source directory path")
                .required(true)
                .index(1)
            )
            .get_matches();

        // You can check the value provided by positional arguments, or option arguments
        if let Some(i) = matches.value_of("DIR") {
            println!("Value for DIR: {}", i);
        }

        matches
            .value_of("DIR")
            .ok_or(CliError::WrongArgs)
            .and_then(check)
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
            Err(_e) => Err(CliError::NotDir(
                dst.canonicalize().unwrap().display().to_string(),
            )),
        }
    }

    match parse_arg_with_clap() {
        Ok(srcdir) => {
            // validate the destination directory
            let dstdir = Path::new(".");
            // error unless current dir (i.e. target) is empty
            if let Ok(empty) = dir_is_empty(&dstdir) {
                if !empty {
                    return Err(Box::from(CliError::DstDirNotEmpty(
                        dstdir.canonicalize()?.display().to_string(),
                    )));
                }
            }

            let dir = Path::new(&srcdir);
            recurse(&dir, &dstdir).or_else(|_| {
                Err(Box::from(CliError::NotDir(
                    dir.canonicalize()?.display().to_string(),
                )))
            })
        }
        Err(e) => Err(Box::from(e)),
    }
}
