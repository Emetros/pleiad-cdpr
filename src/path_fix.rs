use std::{
    fs::{self, create_dir_all, remove_dir_all, remove_file},
    path::Path,
};

use glob::{MatchOptions, glob_with};

wit_bindgen::generate!({ world: "plugin" });

struct Plugin;

impl Guest for Plugin {
    fn restructure(path: String) {
        std::env::set_current_dir(path.clone()).unwrap();
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

            let paths = paths.unwrap();

            // TODO: Add a path count to prevent files from being merged in case some modders added
            // folders such as EBB, Vanilla... Paths.count() takes the ownership and clone() does not
            // exist for Paths

            for glob_path in paths {
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
                if (path.clone() + &source).contains(&(path.clone() + &format!("/{keyword}/"))) {
                    continue;
                };

                let mut dest: String = path.clone();
                let mut modified_src = path.clone();

                // All the lowercase allows to do two things:
                //   - Fix the path in case there is an issue with case sensitive paths
                //   - Prevents ending up with both Assets and assets directories in the game
                //     files because I can't make sure they will end up detected in Proton
                let mut split_src = source.split("/");
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
                    } else if modified_src == path.clone() {
                        modified_src.push_str(&("/".to_string() + part));
                    }
                }

                let abs_src = path.clone() + &source;

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
            for found_path in paths {
                let rel_path = found_path.unwrap();
                let rel_path = rel_path.strip_prefix("../Mod Content").unwrap();
                let abs_path = path.clone() + "/" + rel_path.to_str().unwrap();
                let ignore_file = Path::new(&abs_path);
                if ignore_file.is_file() {
                    if let Err(e) = remove_file(ignore_file) {
                        eprintln!("[Err] unable to delete the file {abs_path}: {e}");
                    };
                }
            }
        }
    }

    fn pre_hook() {
        println!("No pre-hook");
    }

    fn post_hook() {
        println!("No post-hook");
    }
}

export!(Plugin);
