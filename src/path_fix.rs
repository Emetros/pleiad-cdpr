use std::{
    ffi::{CString, c_char},
    fs,
};

use glob::{MatchOptions, glob_with};

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
    let paths_patterns = vec!["archive", "red4ext", "bin"];

    for keyword in paths_patterns {
        let options = MatchOptions {
            case_sensitive: false,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        let pattern_path = path.clone() + &format!("**/{keyword}/**/*");
        let paths = glob_with(&pattern_path, options);
        if let Err(e) = paths {
            eprintln!("[Err] Plugin name: Unable to match pattern: {e}");
            continue;
        }

        for glob_path in paths.unwrap() {
            if let Err(e) = glob_path {
                eprintln!("[Err] Could not retrieve the glob path: {e}");
                continue;
            }

            let glob_path = glob_path.unwrap();
            let source = glob_path
                .into_os_string()
                .into_string()
                .expect("OsString Should be convertible to String type");

            let mut split_src = source.split("/");
            let mut dest: String = path.clone();

            // All the lowercase allows to do two things:
            //   - Fix the path in case there is an issue with case sensitive paths
            //   - Linux is case sensitive, windows isn´t. This prevents ending up with
            //     both Assets and assets directories in the game files because I can't
            //     make sure they will end up detected in Proton
            while let Some(part) = split_src.next() {
                if part.to_lowercase() == keyword.to_lowercase() {
                    dest.push_str(&(keyword.to_string() + "/"))
                } else if dest
                    .to_lowercase()
                    .contains(&format!("/{keyword}/").to_lowercase())
                {
                    dest.push_str(&(part.to_string() + "/"));
                }
            }

            if let Err(e) = fs::copy(&source, &dest) {
                eprintln!("[Err] Could not move {source} to {dest}: {e} | Skipping");
                continue;
            };

            if fs::remove_dir_all(&source).is_ok() {
                println!("[Info] Moved {} to {}", source, dest);
            } else {
                //TODO: gracefully handle this
                eprintln!("[Err] could not delete {source}");
            };
        }
    }
    let readme_path = path + "/**/*.txt";
    let _ignore_paths = vec![readme_path];
}
