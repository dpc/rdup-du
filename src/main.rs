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
	fn lt(&self, other: &SizeSortedFile) -> bool {
		self.size < other.size
	}
}

impl PartialEq for SizeSortedFile {
	fn eq(&self, other: &SizeSortedFile) -> bool {
		self.size == other.size
	}
}

impl Eq for SizeSortedFile {
}

fn is_link(p : &Path) -> bool {
	match lstat(p).unwrap().kind {
		TypeSymlink => true,
		_ => false,
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

fn bytes_to_str(size : u64) -> String {
	format!("{:>12}", size)
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

fn visit_dirs_summary(dir: &Path, size_f : |u64| -> String) {
	let mut entries = TreeSet::<SizeSortedFile>::new();

	match fs::readdir(dir) {
		Ok(dir) => {
			for entry in dir.iter() {
				let size = if !is_link(entry) && entry.is_dir() {
					let mut size = 0;
					visit_dirs(entry, |p| if p.is_file() {

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
	println!("-h --help\tUsage");
}

fn main() {
	let args: Vec<String> = os::args();

	let program = args.get(0).clone();

	let opts = [
		optflag("b", "bytes", "print size in bytes"),
		optflag("h", "help", "print this help menu")
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
		(*matches.free.get(0)).clone()
	} else {
		".".to_string()
	};

	let d = &std::path::Path::new(dir);

	if matches.opt_present("b") {
		visit_dirs_summary(d, bytes_to_str);
	} else {
		visit_dirs_summary(d, bytes_to_humanreadable);
	}
}
