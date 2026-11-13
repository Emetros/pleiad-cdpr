use std::{
    ffi::{CString, c_char},
    fs,
};

use glob::glob;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn restructure(path: *const c_char) {
    let c_path: CString;
    unsafe {
        // Please use from_raw() as we are passing a CString.into_raw() instead of a pointer.
        // This is an arbitrary choice because the CString has been built for this specific method
        // and wont be used in the future.
        c_path = CString::from_raw(path as *mut _);
    }
    let path = c_path.to_str().unwrap().to_string();

    // We then define the common standard patterns. This can also be handled by an external file such
    // as a json or a toml to provide a standard plugin that allows easy modification/maintenance by
    // the user.
    let bin_path = path.clone() + "/bin/**";
    let archive_path = path.clone() + "/archive/**";
    let red4ext_path = path.clone() + "/red4ext/**";
    let paths_patterns = vec![archive_path, bin_path, red4ext_path];

    let readme_path = path + "/**.txt";
    let _ignore_paths = vec![readme_path];

    for path in paths_patterns {
        let paths = glob(&path);
        if let Err(e) = paths {
            eprintln!("[Err] Plugin name: Unable to match pattern: {e}");
            continue;
        }

        let glob_paths = paths.unwrap();

        for glob_path in glob_paths {
            // We should remove the first part of the path until it starts with either bin or
            // archive then copy paste it to the current directory
            let source = glob_path.unwrap().into_os_string().into_string().unwrap();
            // TODO: build the dest
            let _split_src = source.split("/");
            // Add a while loop to iterate until source is the desired path, i.e. archive, red4ext...
            let dest = "";

            if let Err(e) = fs::copy(&source, dest) {
                eprintln!("[Err] Could not move a file: {e} | Skipping");
                continue;
            };

            if fs::remove_dir_all(&source).is_ok() {
                println!("[Info] Moved a file from {} to {}", source, dest);
            } else {
                //TODO: gracefully handle this
                eprintln!("[Err] could not delete the source");
            };
        }
    }
}
