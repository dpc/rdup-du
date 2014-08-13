extern crate getopts;
extern crate collections;

use collections::treemap::TreeSet;
use getopts::{optflag,getopts};
use std::cmp::{Ord,Eq,PartialEq,PartialOrd,Ordering,Less,Equal,Greater};
use std::io;
use std::io::fs;
use std::io::fs::lstat;
use std::io::IoError;
use std::io::stdio::stderr;
use std::io::TypeSymlink;
use std::num::pow;
use std::os;
use std::path::Path;

struct SizeSortedFile {
	entry : Path,
	size : u64,
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

impl Eq for SizeSortedFile {
}

fn is_link(p : &Path) -> Result<bool,IoError> {
	lstat(p).map(|e| match e.kind {
		TypeSymlink => true,
		_ => false,
	})
}

// Is the entry a dir, that we should recursively traverse?
fn should_visit(p : &Path, fs : Option<u64>) -> bool {
	if !is_link(p).unwrap_or_else(
		|e| {print_error_path(p, e); false}
		) && p.is_dir() {
		if fs.is_some() {
			let stat = match fs::stat(p) {
				Err(e) => {
					print_error_path(p, e);
					return false;
				},
				Ok(stat) => stat
			};
			if fs != Some(stat.unstable.device) {
				false
			} else {
				true
			}
		} else {
			true
		}
	} else {
		false
	}
}

fn bytes_to_humanreadable(size : u64) -> String {
	let postfixes = ["B", "K", "M", "G", "T"];

	for (i, postfix) in postfixes.iter().enumerate() {
		if size < pow(1000u64, i+1) {
			return format!("{:>7}{}", size / pow(1000u64, i), postfix)
		}
	}

	let i = postfixes.len() - 1;
	format!("{}{}", size / pow(1024u64, i), postfixes[i])
}

fn bytes_to_string(size : u64) -> String {
	format!("{:>12}", size)
}

fn visit_dirs(dir: &Path, fs : Option<u64>, cb: |&Path|, err: |&Path, IoError|) {
	let mut nobackup = dir.clone();
	nobackup.push(".nobackup");

	if nobackup.is_file() {
		cb(&nobackup);
		return;
	}

	if dir.is_dir() {
		let contents = fs::readdir(dir);
		match contents {
			Ok(contents) => {
				for entry in contents.iter() {
					if should_visit(entry, fs) {
						visit_dirs(entry, fs, |p| cb(p), |p, e| err(p, e));
					} else {
						cb(entry);
					}
				}
			},
			Err(e) => {
				err(dir, e);
			}
		};
	} else {
		err(dir, io::standard_error(io::InvalidInput));
	}
}

fn print_error_path(p : &Path, e : IoError) {
	(writeln!(&mut stderr(), "{}: {}", p.display(), e)).unwrap();
}

fn visit_dirs_summary(dir: &Path, config : &CmdConfig) {
	let mut entries = TreeSet::<SizeSortedFile>::new();

        let size_f = match config.bytes {
            true => bytes_to_string,
            false => bytes_to_humanreadable
        };

	let fs = if config.localfs {
		match fs::stat(dir) {
			Err(e) => {
				print_error_path(dir, e);
				return;
			},
			Ok(stat) => Some(stat.unstable.device)
		}
	} else { None };

	match fs::readdir(dir) {
		Ok(dir) => {
			for entry in dir.iter() {


				let size = if should_visit(entry, fs) {
					let mut size = 0;
					visit_dirs(entry, fs, |p| if p.is_file() {

						lstat(p)
							.map(|l| size = size + l.size)
							.unwrap_or_else(
								|e| {print_error_path(p, e)}
								)
						},
						print_error_path
					);
					size
				} else {
					lstat(entry)
						.map(|l| l.size)
						.unwrap_or_else(
							|e| {print_error_path(entry, e); 0}
							)
				};
				entries.insert(SizeSortedFile{entry: entry.clone(), size: size});
			}
		},
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
	localfs : bool,
	bytes : bool
}

fn main() {
	let args: Vec<String> = os::args();

	let program = args[0].clone();

	let opts = [
		optflag("b", "bytes", "print size in bytes"),
		optflag("h", "help", "print this help menu"),
		optflag("x", "localfs", "stay on local filesystem"),
			];

	let matches = match getopts(args.tail(), opts) {
		Ok(m) => { m }
		Err(f) => { fail!(f.to_err_msg()) }
	};
	if matches.opt_present("h") {
		print_usage(program.as_slice());
		return;
	}
	let dir = if !matches.free.is_empty() {
		matches.free[0].clone()
	} else {
		".".to_string()
	};

	let d = &std::path::Path::new(dir);

	let conf = CmdConfig {
		bytes: matches.opt_present("b"),
		localfs: matches.opt_present("x"),
	};

	visit_dirs_summary(d, &conf);
}
// vim: ts=8 sw=8 noexpandtab
