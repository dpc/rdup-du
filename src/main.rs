extern crate getopts;
extern crate libc;

use std::collections::BTreeSet;
use getopts::Options;
use std::cmp::{Ord, Eq, PartialEq, PartialOrd, Ordering};
use std::cmp::Ordering::{Less, Equal, Greater};
use std::io;
use std::io::Write;
use std::fs;
use std::io::stderr;
use std::path::Path;
use std::path;
use std::os::linux::fs::MetadataExt;

struct SizeSortedFile {
    entry: path::PathBuf,
    size: u64,
}

impl Ord for SizeSortedFile {
    fn cmp(&self, other: &SizeSortedFile) -> Ordering {
        if self.size < other.size {
            Less
        } else if self.size > other.size {
            Greater
        } else {
            Equal
        }
    }
}


impl PartialOrd for SizeSortedFile {
    fn partial_cmp(&self, other: &SizeSortedFile) -> Option<Ordering> {
        if self.size < other.size {
            Some(Less)
        } else if self.size > other.size {
            Some(Greater)
        } else {
            Some(Equal)
        }
    }
}

impl PartialEq for SizeSortedFile {
    fn eq(&self, other: &SizeSortedFile) -> bool {
        self.size == other.size
    }
}

impl Eq for SizeSortedFile {}

// Is the entry a dir, that we should recursively traverse?
fn should_visit(p: &Path, fs: Option<u64>) -> bool {
    match p.metadata() {
        Err(e) => {
            print_error_path(p, e);
            false
        }
        Ok(meta) => {
            if meta.file_type().is_symlink() {
                false
            } else if !meta.file_type().is_dir() {
                false
            } else if fs.is_some() {
                fs == Some(meta.st_dev())
            } else {
                true
            }
        }
    }
}

fn bytes_to_humanreadable(size: u64) -> String {
    let postfixes = ["B", "K", "M", "G", "T"];

    let mut size = size;
    for postfix in postfixes.iter() {
        if size < 1000u64 {
            return format!("{:>7}{}", size, postfix);
        }

        size /= 1000u64;
    }

    let i = postfixes.len() - 1;
    format!("{}{}", size, postfixes[i])
}

fn bytes_to_string(size: u64) -> String {
    format!("{:>12}", size)
}

fn visit_dirs(dir: &Path,
              fs: Option<u64>,
              cb: &mut FnMut(&Path),
              mut err: &mut FnMut(&Path, io::Error)) {
    let nobackup = dir.clone();
    nobackup.join(".nobackup");

    if nobackup.is_file() {
        cb(nobackup);
        return;
    }

    if dir.is_dir() {
        let contents = fs::read_dir(dir);
        match contents {
            Ok(contents) => {
                for entry in contents {
                    match entry {
                        Ok(entry) => {
                            let entry_path = entry.path();
                            if should_visit(&entry_path, fs) {
                                visit_dirs(&entry_path, fs, &mut |p| cb(p), &mut |p, e| err(p, e));
                            } else {
                                cb(&entry_path);
                            }
                        }
                        Err(e) => err(dir, e),
                    }
                }
            }
            Err(e) => {
                err(dir, e);
            }
        };
    } else {
        err(dir,
            io::Error::new(io::ErrorKind::InvalidInput, dir.to_str().unwrap_or("")));
    }
}

fn print_error_path(p: &Path, e: io::Error) {
    (writeln!(&mut stderr(), "{}: {}", p.to_string_lossy(), e)).unwrap();
}

fn visit_dirs_summary(dir: &Path, config: &CmdConfig) {
    let mut entries = BTreeSet::<SizeSortedFile>::new();

    let size_f: fn(u64) -> String = match config.bytes {
        true => bytes_to_string,
        false => bytes_to_humanreadable,
    };

    let fs = if config.localfs {
        match dir.metadata() {
            Err(e) => {
                print_error_path(dir, e);
                return;
            }
            Ok(stat) => Some(stat.st_dev()),
        }
    } else {
        None
    };

    match dir.read_dir() {
        Ok(dir_listing) => {
            for entry in dir_listing {
                match entry {
                    Ok(entry) => {
                        let entry_path = entry.path();
                        let size = if should_visit(&entry_path, fs) {
                            let mut size = 0;
                            visit_dirs(&entry_path,
                                       fs,
                                       &mut |p| if p.is_file() {
                                           match p.metadata() {
                                               Err(e) => print_error_path(p, e),
                                               Ok(meta) => size = size + meta.len(),
                                           }
                                       },
                                       &mut print_error_path);
                            size
                        } else {
                            match entry.metadata() {
                                Err(e) => {
                                    print_error_path(&entry_path, e);
                                    0
                                }
                                Ok(meta) => meta.len(),
                            }
                        };
                        entries.insert(SizeSortedFile {
                            entry: entry_path.to_path_buf(),
                            size: size,
                        });
                    }
                    Err(e) => print_error_path(dir, e),
                }
            }
        }
        Err(err) => {
            print_error_path(dir, err);
        }
    };

    let mut total_size = 0;
    for entry in entries.iter() {
        println!("{:>7} {}", size_f(entry.size), entry.entry.display());
        total_size = total_size + entry.size;
    }

    println!("");
    println!("{:>7} total", size_f(total_size));
}

fn print_usage(program: &str) {
    println!("Usage: {} [options]", program);
    println!("-b --bytes\tPrint size in bytes");
    println!("-x --localfs\tStay on local filesystem");
    println!("-h --help\tUsage");
}

struct CmdConfig {
    localfs: bool,
    bytes: bool,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("b", "bytes", "print size in bytes");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("x", "localfs", "stay on local filesystem");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.opt_present("h") {
        print_usage(&program);
        return;
    }

    let dir = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        ".".to_string()
    };

    let d = &std::path::Path::new(&dir);

    let conf = CmdConfig {
        bytes: matches.opt_present("b"),
        localfs: matches.opt_present("x"),
    };

    visit_dirs_summary(d, &conf);
}
