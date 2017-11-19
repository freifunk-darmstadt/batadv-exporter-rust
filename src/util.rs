use std::env;
use std::path::{Path, PathBuf};

pub fn which<P>(exe_name: P) -> bool
where
    P: AsRef<Path>,
{
    env::var_os("PATH")
        .and_then(|paths| {
            env::split_paths(&paths)
                .filter_map(|dir| {
                    let full_path = dir.join(&exe_name);
                    if full_path.is_file() { Some(()) } else { None }
                })
                .next()
        })
        .is_some()
}
