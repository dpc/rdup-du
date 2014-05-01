extern crate getopts;
extern crate collections;

use collections::treemap::TreeSet;
use getopts::{optopt,optflag,getopts,OptGroup};
use std::cmp::{Ordering,Less,Equal,Greater};
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

impl TotalOrd for SizeSortedFile {
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

impl Ord for SizeSortedFile {
	fn lt(&self, other: &SizeSortedFile) -> bool {
		self.size < other.size
	}
}

impl TotalEq for SizeSortedFile {
}

impl Eq for SizeSortedFile {
	fn eq(&self, other: &SizeSortedFile) -> bool {
		self.size == other.size
	}
}

fn is_link(p : &Path) -> bool {
	match lstat(p).unwrap().kind {
		TypeSymlink => true,
		_ => false,
	}
}

fn format_size(size : u64) -> ~str {
	let postfixes = ["B", "K", "M", "G", "T"];

	for (i, postfix) in postfixes.iter().enumerate() {
		if size < pow(1024u64, i+1) {
			return format!("{:>7}{}", size / pow(1024u64, i), postfix)
		}
	}

	let i = postfixes.len() - 1;
	format!("{:>6}{}", size / pow(1024u64, i), postfixes[i])
}

fn visit_dirs(dir: &Path, cb: |&Path|, err: |&Path, IoError|) {
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
					if !is_link(entry) && entry.is_dir() {
						visit_dirs(entry, |p| cb(p), |p, e| err(p, e));
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

fn visit_dirs_summary(dir: &Path) {
	let mut entries = TreeSet::<SizeSortedFile>::new();

	match fs::readdir(dir) {
		Ok(dir) => {
			for entry in dir.iter() {
				let size = if !is_link(entry) && entry.is_dir() {
					let mut size = 0;
					visit_dirs(entry, |p| if p.is_file() {
						match lstat(p) {
							Ok(lstat) => {
								size = size + lstat.size;
							},
							Err(err) => {
								print_error_path(entry, err);
							}
						}
					},
					print_error_path
					);
					size
				} else {
					match lstat(entry) {
							Ok(lstat) => {
								lstat.size
							},
							Err(err) => {
								print_error_path(entry, err);
								0
							}
						}
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
		println!("{} {}", format_size(entry.size), entry.entry.display());
		total_size = total_size + entry.size;
	}

	println!("");
	println!("{} total", format_size(total_size));
}

fn print_usage(program: &str, _opts: &[OptGroup]) {
	println!("Usage: {} [options]", program);
	println!("-b\tPrint size in bytes");
	println!("-h --help\tUsage");
}

fn main() {
	let args = os::args();
	let program = args[0].clone();

	let opts = [
		optopt("b", "", "print size in bytes", "NAME"),
		optflag("h", "help", "print this help menu")
			];

	let matches = match getopts(args.tail(), opts) {
		Ok(m) => { m }
		Err(f) => { fail!(f.to_err_msg()) }
	};
	if matches.opt_present("h") {
		print_usage(program, opts);
		return;
	}
	if matches.opt_present("b") {
		fail!("not implemented");
	}
	let dir = if !matches.free.is_empty() {
		(*matches.free.get(0)).clone()
	} else {
		~"."
	};

	visit_dirs_summary(&std::path::Path::new(dir));
}
