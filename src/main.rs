const USAGE: &str = "
Usage:
    graft {PATH}";

use std::env;
use std::error;
use std::fmt;

// We derive `Debug` because all types should probably derive `Debug`.
// This gives us a reasonable human-readable description of `CliError` values.
#[derive(Debug)]
enum CliError {
    MissingArg,
    NotDir,
    WrongArgs,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
            CliError::MissingArg => write!(f, "Missing Arg:"),
            CliError::NotDir => write!(f, "Not a Directory:"),
            CliError::WrongArgs => write!(f, "Wrong number of args"),
        }
    }
}

impl error::Error for CliError {
    fn cause(&self) -> Option<&error::Error> {
        Some(self)
    }
}

fn main() -> Result<(), Box<std::error::Error>> {
    use std::ffi::OsString;
    use std::path::Path;

    fn parse_arg(mut argv: env::ArgsOs) -> Result<OsString, CliError> {
        fn check(p: OsString) -> Result<OsString, CliError> {
            // TODO: is the dir Readable?  eXecutable?
            if Path::new(&p.clone()).is_dir() {
                Ok(p)
            } else {
                Err(CliError::NotDir)
            }
        }

        let dir = argv
            .by_ref()
            .nth(1)
            .ok_or(CliError::MissingArg)
            .and_then(check);

        if argv.count() != 0 {
            print!("{}\n\n", USAGE);
            return Err(CliError::WrongArgs);
        };

        dir
    }

    // 1. create subdirectory tree
    // 2. if not symlink, create symlink
    // 3. if symlink, copy symlink (both absolute and relative)

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
                let _a = fs::create_dir(&target);

                // then decend
                let _b = recurse(&entry_path, &target);
            } else if !ft_data.is_symlink() {
                // create symlink
                let _a = unix_fs::symlink(&entry_path, &target);
            } else if ft_data.is_symlink() {
                // copy symlink
                let _a = unix_fs::symlink(&fs::read_link(entry_path)?, &target);
            }
        }

        // TODO: error handling?!?!?!
        Ok(())
    }

    let dstdir = Path::new(".");

    fn dir_is_empty(dst: &Path) -> Result<bool, bool> {
        match dst.read_dir() {
            Ok(entry) => {
                if entry.count() != 0 {
                    println!("Directory not empty!!!");
                    Err(false)
                } else {
                    Ok(true)
                }
            }
            Err(_e) => Err(false),
        }
    }

    // error unless current dir (i.e. target) is empty
    assert_eq!(dir_is_empty(&dstdir), Ok(true));

    match parse_arg(env::args_os()) {
        Ok(srcdir) => {
            let dir = Path::new(&srcdir);
            recurse(&dir, &dstdir).or(Err(Box::new(CliError::NotDir)))
        }
        Err(e) => return Err(Box::new(e)),
    }
}
