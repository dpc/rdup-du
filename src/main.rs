extern crate getopts;
extern crate collections;

use collections::treemap::TreeSet;
use getopts::{optopt,optflag,getopts,OptGroup};
use std::cmp::{Ordering,Less,Equal,Greater};
use std::io;
use std::io::fs;
use std::io::fs::lstat;
use std::os;
use std::path::Path;
use std::num::pow;
use std::io::TypeSymlink;

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

fn visit_dirs(dir: &Path, cb: |&Path|) -> io::IoResult<()> {
	let mut nobackup = dir.clone();
	nobackup.push(".nobackup");

	if nobackup.is_file() {
		cb(&nobackup);
		return Ok(())
	}

	if dir.is_dir() {
		let contents = try!(fs::readdir(dir));
		for entry in contents.iter() {
			if !is_link(entry) && entry.is_dir() {
				match visit_dirs(entry, |p| cb(p)) {
					Err(x) => return Err(x),
					Ok(_) => {},
				};
			} else {
				cb(entry)
			}
		}
		Ok(())
	} else {
		Err(io::standard_error(io::InvalidInput))
	}
}


fn visit_dirs_summary(dir: &Path) {
	let mut entries = TreeSet::<SizeSortedFile>::new();

	for entry in fs::readdir(dir).unwrap().iter() {
		let size = if !is_link(entry) && entry.is_dir() {
			let mut size = 0;
			visit_dirs(entry, |p| if p.is_file() {
				size = size + lstat(p).unwrap().size
			} ).unwrap();
			size
		} else {
			lstat(entry).unwrap().size
		};

		entries.insert(SizeSortedFile{entry: entry.clone(), size: size});
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
