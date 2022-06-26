extern crate clap;
extern crate which;
extern crate tempfile;
#[macro_use]
extern crate itertools;
mod args;
mod library;
mod create;
mod list;
mod extract;

use crate::library::*;
use crate::args::*;
use crate::create::create;
use crate::list::list;
use crate::extract::extract;

fn main() {

	let args = Arguments::parse();

	match args.command {
		ArcCmd::Create(d)  =>  create(d),
		ArcCmd::List(d)    =>    list(d),
		ArcCmd::Extract(d) => extract(d),
		ArcCmd::Help => {},
		_ => {
			error("この機能は未実装です");
		}
	}

}
