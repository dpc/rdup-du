# RDUP Disk Usage

[![Build Status](https://travis-ci.org/dpc/rdu.svg?branch=master)](https://travis-ci.org/dpc/rdu)

## Introduction

`rdu` is a simple tools that will estimate the disk usage of files that will be backed up by [rdup backup utility][1]

In an essence it just traverse filesystem, omitting directories containing `.nobackup` file and calculate summary of space usage. It's useful for identifying files and directories that take the most space.

  [1]: https://github.com/miekg/rdup

## Implementation

`rdu` is written [Rust programming Language][2]

  [2]: http://rust-lang.org

## Building & installation

	make
	cp bin/rdu /usr/bin/

## Usage

	rdu <directory>
