const USAGE: &str = "
Usage:
    graft {PATH}";

fn main() -> Result<(), Box<std::error::Error>> {

    use std::path::Path;
    use std::ffi::OsString;

    fn get_arg() -> Result<OsString, String> {
        use std::env;

        if env::args_os().count() != 2 {
            print!("{}\n\n", USAGE);
            return Err("Expecting exactly 1 argument".to_string())
        }

        match env::args_os().nth(1) {
            Some(ostr) => {
                let foo = ostr.clone();
                // check if {PATH} exists
                if Path::new(&foo).is_dir() {
                    print!("Good dir\n");
                    return Ok(ostr)
                } else {
                    match ostr.into_string() {
                        Ok(t) => return Err(t + " is not a directory!"),
                        Err(_e) => return Err("Bad ostr".to_string())
                    }
                }
            },
            None => {
                print!("{}\n\n", USAGE);
                return Err("Expected exactly 1 argument".to_string())
            }
        }

        env::args_os().nth(1)
            .ok_or(Err("Expecting exactly 1 argument"))
            .map( |ostr| {
                let foo = ostr.clone();
                if Path::new(&foo).is_dir() {
                    ostr
                } else {
                    unimplemented!
                }
            }
            )

    }

    // 1. create subdirectory tree
    // 2. if not symlink, create symlink
    // 3. if symlink, copy symlink (both absolute and relative)

    fn recurse(srcdir: &Path, cwd: &Path) -> std::io::Result<()> {
        use std::fs;
        use std::os::unix::fs as unix_fs;

        for wrapped_entry in try!(fs::read_dir(srcdir)) {

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
                    print!("Directory not empty!!!");
                    Err(false)
                } else {
                    Ok(true)
                }
            },
            Err(_e) => Err(false)
        }
    }

    // error unless current dir (i.e. target) is empty
    assert_eq!(dir_is_empty(&dstdir), Ok(true));

    match get_arg() {
        Ok(srcdir) => {
            let dir = Path::new(&srcdir);
            recurse(&dir, &dstdir).unwrap();
            Ok(())
        },
        Err(e) => return Err(e.into())
    }
}