use std::{
    ffi::{CStr, CString, c_char},
    fs::{self, create_dir_all, remove_dir_all, remove_file},
    mem,
    path::Path,
};

use glob::{MatchOptions, glob_with};

#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: usize) -> *const u8 {
    let mut string_buf: Vec<u8> = vec![0; len];
    let buf_ptr = string_buf.as_mut_ptr();
    mem::forget(string_buf);
    buf_ptr
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    unsafe {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_compatible_games() -> *const c_char {
    let str = "Cyberpunk 2077".to_string();
    let c_str = CString::new(str).expect("Should be convertible");
    c_str.as_ptr()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn restructure(path: *const c_char) {
    println!("=========== BEGINNING RESTRUCT ===========");
    let c_path: String;
    unsafe {
        c_path = CStr::from_ptr(path).to_str().unwrap().to_owned();
    }

    std::env::set_current_dir(c_path.clone()).unwrap();
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    // We define the common standard directories. This could also be handled by an external file such
    // as a json or a toml to provide a standard plugin that allows easy modification/maintenance by
    // the users instead of forcing a plugin compilation.
    let paths_patterns = vec!["archive", "red4ext", "bin", "r6"];

    let mut directories: Vec<String> = Vec::new();
    let mut files: Vec<(String, String)> = Vec::new();

    for keyword in paths_patterns {
        // Set a starting point from a relative path since we are in a sandboxed
        // environment
        let rel_pattern = &format!("../**/{keyword}/**/*");
        let paths = glob_with(&rel_pattern, options);

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
                .clone()
                .into_os_string()
                .into_string()
                .expect("OsString Should be convertible to String type")
                .strip_prefix("../Mod Content")
                .expect("../Mod Content/ shoud be in the path as it has been manually added")
                .to_string();

            // Prevent any useless calculation if the source already matches the desired
            // pattern
            if (c_path.clone() + &source).contains(&(c_path.clone() + &format!("/{keyword}/"))) {
                continue;
            };

            // We now rebuild the path from the one we found from glob
            // All the lowercase allows to do two things:
            //   - Fix the path in case there is an issue with case sensitive paths
            //   - Prevents ending up with both Assets and assets directories in the game
            //     files because I can't make sure they will end up detected in Proton
            let mut split_src = source.split("/");
            let mut dest: String = c_path.clone();
            let mut modified_src = c_path.clone();
            while let Some(part) = split_src.next() {
                if part.is_empty() {
                    continue;
                }
                if part.to_lowercase() == keyword.to_lowercase() {
                    dest.push_str(&("/".to_string() + keyword))
                } else if dest
                    .to_lowercase()
                    .contains(&format!("/{keyword}").to_lowercase())
                {
                    dest.push_str(&("/".to_string() + part));
                } else if modified_src == c_path.clone() {
                    modified_src.push_str(&("/".to_string() + part));
                }
            }

            let abs_src = c_path.clone() + &source;

            if Path::new(&abs_src).is_dir() {
                if !directories.contains(&modified_src) {
                    directories.push(modified_src);
                }
            } else {
                files.push((abs_src.clone(), dest.clone()));
            }
        }
    }

    for (src, dest) in files {
        if !Path::new(&src).exists() {
            continue;
        }

        let dest_dir = Path::new(&dest).parent().expect("Should have a parent");
        if !Path::new(&dest_dir).exists() {
            create_dir_all(&dest_dir).expect("Should be allowed to create dir");
        }

        if let Err(e) = fs::copy(&src, &dest) {
            eprintln!("[Err] Could not move {} to {dest}: {e} | Skipping", src);
            continue;
        };

        if fs::remove_file(&src).is_ok() {
            println!("[Info] Moved {} to {}", src, dest);
        } else {
            //TODO: gracefully handle this
            eprintln!("[Err] could not delete the file {}", src);
        };
    }

    directories.sort_by(|first, next| next.len().cmp(&first.len()));

    for directory in directories {
        if Path::new(&directory).exists() {
            if remove_dir_all(&directory).is_err() {
                eprintln!("[Err] Directory not empty -- Skipping: {}", directory);
            };
        }
    }

    // These files will be deleted after extraction, which prevents unnecessary listing later and avoidable meaningless
    // conflicts reported in the UI
    let ignore_extensions = vec!["txt"];
    for keyword in ignore_extensions {
        let rel_pattern = format!("../**/*.{keyword}");
        let paths = glob_with(&rel_pattern, options);

        if paths.is_err() {
            continue;
        }
        let paths = paths.unwrap();
        for path in paths {
            let path = path.unwrap();
            let path = path.strip_prefix("../Mod Content").unwrap();
            let abs_path = c_path.clone() + "/" + path.to_str().unwrap();
            let ignore_file = Path::new(&abs_path);
            if ignore_file.is_file() {
                if let Err(e) = remove_file(ignore_file) {
                    eprintln!("[Err] unable to delete the file {abs_path}: {e}");
                };
            }
        }
    }
    println!("============== END RESTRUCT ==============");
}
