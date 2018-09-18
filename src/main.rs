fn main() {

    use std::path::Path;
    use std::ffi::OsString;

    fn get_arg() -> Result<OsString, String> {
        use std::env;

        let mut arg: OsString = OsString::from("/dev/null");

        if env::args_os().count() != 2 {
            return Err("asdf".to_string())
        }

        // Prints each argument on a separate line
        for argument in env::args_os() {
            println!("{:?}", argument);
            arg = argument;
        }
        Ok(arg)
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
            let ft_data = entry.metadata().unwrap().file_type();

            let target = cwd.canonicalize().unwrap().join(entry.file_name());

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

        Ok(())
    }

    let srcdir = get_arg().unwrap();
    let dstdir = Path::new(".");
    let dir = Path::new(&srcdir);
    recurse(&dir, &dstdir).unwrap()

}