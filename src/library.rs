use std::process::{Command,Stdio,exit};
use std::fmt::Display;
use std::convert::AsRef;
pub use std::ffi::{OsStr,OsString};
use std::env::current_dir;
use std::io::Write;
pub use std::path::{Path,PathBuf};
use which::which;
pub use tempfile::{tempdir,TempDir};
use crate::args::*;

pub type VS = Vec<OsString>;

pub fn vs<I,S:Display>(arr:I) -> VS where I:IntoIterator<Item=S>, S:AsRef<OsStr> {
	return arr.into_iter().map( |s| OsString::from(s.to_string()) ).collect();
}

pub fn error<S:Display>(message:S) where S:AsRef<OsStr> {
	eprintln!("{}",message);
	exit(1);
}

pub struct Cmd {
	pub prog: OsString,
	pub args: VS,
	pub cwd: PathBuf,
	pub env: Vec<(OsString,OsString)>,
	pub stdin: Option<String>,
	pub inherit_output: bool,
	pub stdout: Option<String>,
	pub stderr: Option<String>
}
impl Cmd {
	pub fn new<S:Display>(prog:S,args:VS) -> Self where S:AsRef<OsStr> {
		return Cmd {
			prog: OsString::from(prog.to_string()),
			args: args,
			cwd: current_dir().expect("カレントディレクトリが存在しません"),
			env: vec![],
			stdin: None,
			inherit_output: true,
			stdout: None,
			stderr: None
		}
	}
	pub fn new_cwd<S:Display>(prog:S,args:VS,cwd:PathBuf) -> Self where S:AsRef<OsStr> {
		return Cmd {
			prog: OsString::from(prog.to_string()),
			args: args,
			cwd: cwd,
			env: vec![],
			stdin: None,
			inherit_output: true,
			stdout: None,
			stderr: None
		}
	}
	pub fn new_cwd_stdin<S:Display>(prog:S,args:VS,cwd:PathBuf,stdin:String) -> Self where S:AsRef<OsStr> {
		return Cmd {
			prog: OsString::from(prog.to_string()),
			args: args,
			cwd: cwd,
			env: vec![],
			stdin: Some(stdin),
			inherit_output: true,
			stdout: None,
			stderr: None
		}
	}
	pub fn new_cwd_env<I,S:Display>(prog:S,args:VS,cwd:PathBuf,env:I) -> Self where I:IntoIterator<Item=(S,S)>,S:AsRef<OsStr> {
		return Cmd {
			prog: OsString::from(prog.to_string()),
			args: args,
			cwd: cwd,
			env: env.into_iter().map(|(k,v)| {
				(
					OsString::from(k.to_string()),
					OsString::from(v.to_string())
				)
			}).collect::<Vec<(OsString,OsString)>>(),
			stdin: None,
			inherit_output: true,
			stdout: None,
			stderr: None
		}
	}
}

pub fn several_cmd<I>(cmd_list:I) -> bool where I:IntoIterator<Item=Cmd> {
	for cmd in cmd_list.into_iter() {
		let p = match which(&cmd.prog) {
			Ok(p) => p,
			Err(_) => {
				error(
					format!(
						"コマンド {} が見つからないので実行できません",
						cmd.prog.to_str().unwrap()
					)
				);
				panic!();
			}
		};
		let mut c = Command::new(p);
		c.args(cmd.args).current_dir(cmd.cwd);
		for t in cmd.env { c.env(t.0,t.1); }
		if cmd.inherit_output { c.stdout(Stdio::inherit()).stderr(Stdio::inherit()); }
		else { c.stdout(Stdio::null()).stderr(Stdio::null()); }
		match cmd.stdin {
			Some(_) => c.stdin(Stdio::piped()),
			None    => c.stdin(Stdio::inherit())
		};
		let mut r = true;
		match c.spawn() {
			Ok(mut child) => {
				match (&cmd.stdin,&mut child.stdin) {
					(Some(s),Some(writer)) => {
						match writer.write_all(s.as_bytes()) {
							Ok(_) => {},
							Err(_) => { r = false }
						}
						match writer.flush() {
							Ok(_) => {},
							Err(_) => { r = false }
						}
					},
					(None,None) => {},
					_ => { r = false }
				}
				match child.wait() {
					Ok(es) => {
						match es.code() {
							Some(0) => {},
							_ => { r = false; }
						}
					},
					Err(_) => { r = false; }
				}
			},
			Err(e) => {
				eprintln!("コマンドの起動に失敗しました: {:?}",e);
				r = false;
			}
		};
		if !r { return false; }
	}
	return true;
}

pub trait TDAddition {
	fn join_str<S>(&self,path:S) -> OsString where S:AsRef<Path>;
}
impl TDAddition for tempfile::TempDir {
	fn join_str<S>(&self,path:S) -> OsString where S:AsRef<Path> {
		return self.path().join(path).into_os_string();
	}
}

pub trait GetAbsolutePath {
	fn absolute_path(&self) -> PathBuf;
}
impl GetAbsolutePath for Path {
	fn absolute_path(&self) -> PathBuf {
		if self.is_relative() {
			let cd = current_dir().expect("カレントディレクトリが存在しません");
			return cd.join(self);
		}
		else { return self.to_path_buf(); }
	}
}



pub fn guess_type(file:&String,create:bool) -> Option<ArcType> {
	macro_rules! tar_compress {
		($compress:expr) => { {
			match create {
				true  => $compress,
				false => ArcType::Tar
			}
		} };
	}
	Some(match file {
		s if s.ends_with(".zip")  => ArcType::Zip,
		s if s.ends_with(".7z")   => ArcType::SevenZ,
		s if s.ends_with(".tar")  => ArcType::Tar,
		s if s.ends_with(".cpio") => ArcType::Cpio,
		s if s.ends_with(".rar")  => ArcType::Rar,
		s if s.ends_with(".aar")  => ArcType::Aar,
		s if s.ends_with(".wim")  => ArcType::Wim,
		s if s.ends_with(".dmg")  => ArcType::Dmg,
		s if s.ends_with(".iso")  => ArcType::Iso,
		s if s.ends_with(".zpaq") => ArcType::Zpaq,
		s if s.ends_with(".lhz")  => ArcType::Lha,
		s if s.ends_with(".tar.Z")                         => tar_compress!(ArcType::Compress),
		s if s.ends_with(".tar.gz") ||s.ends_with(".tgz")  => tar_compress!(ArcType::Gzip),
		s if s.ends_with(".tar.bz2")||s.ends_with(".tbz2") => tar_compress!(ArcType::Bzip2),
		s if s.ends_with(".tar.xz") ||s.ends_with(".txz")  => tar_compress!(ArcType::Xz),
		s if s.ends_with(".tar.lz") ||s.ends_with(".tlz")  => tar_compress!(ArcType::Lzip),
		s if s.ends_with(".tar.lzma")                      => tar_compress!(ArcType::Lzma),
		s if s.ends_with(".tar.lz4")                       => tar_compress!(ArcType::Lz4),
		s if s.ends_with(".tar.lzo")                       => tar_compress!(ArcType::Lzop),
		s if s.ends_with(".tar.lrz")                       => tar_compress!(ArcType::Lrzip),
		s if s.ends_with(".tar.rz")                        => tar_compress!(ArcType::Rzip),
		s if s.ends_with(".tar.zst")||s.ends_with(".tzst") => tar_compress!(ArcType::Zstd),
		s if s.ends_with(".tar.br")                        => tar_compress!(ArcType::Brotli),
		s if s.ends_with(".tar.lzfse")                     => tar_compress!(ArcType::Lzfse),
		s if s.ends_with(".Z")     => ArcType::Compress,
		s if s.ends_with(".gz")    => ArcType::Gzip,
		s if s.ends_with(".bz2")   => ArcType::Bzip2,
		s if s.ends_with(".xz")    => ArcType::Xz,
		s if s.ends_with(".lz")    => ArcType::Lzip,
		s if s.ends_with(".lzma")  => ArcType::Lzma,
		s if s.ends_with(".lz4")   => ArcType::Lz4,
		s if s.ends_with(".lzo")   => ArcType::Lzop,
		s if s.ends_with(".lrz")   => ArcType::Lrzip,
		s if s.ends_with(".rz")    => ArcType::Rzip,
		s if s.ends_with(".zst")   => ArcType::Zstd,
		s if s.ends_with(".br")    => ArcType::Brotli,
		s if s.ends_with(".lzfse") => ArcType::Lzfse,
		_ => { return None; }
	})
}

pub fn compress_ext(at:&ArcType) -> String {
	return match at {
		ArcType::Compress => "Z"    ,
		ArcType::Gzip     => "gz"   ,
		ArcType::Bzip2    => "bz2"  ,
		ArcType::Xz       => "xz"   ,
		ArcType::Lzip     => "lz"   ,
		ArcType::Lzma     => "lzma" ,
		ArcType::Lz4      => "lz4"  ,
		ArcType::Lzop     => "lzo"  ,
		ArcType::Lrzip    => "lrz"  ,
		ArcType::Rzip     => "rz"   ,
		ArcType::Zstd     => "zst"  ,
		ArcType::Brotli   => "br"   ,
		ArcType::Lzfse    => "lzfse",
		_ => { panic!(); }
	}.to_string();
}

pub fn rate_conversion(rate:&mut u8,arc_type:&ArcType) {

	let r = *rate;
	if r>9 {
		error("圧縮率は 0-9 の整数で指定します。");
	}

	*rate = match arc_type {
		ArcType::Zip|ArcType::Gzip|ArcType::Bzip2|ArcType::Xz|ArcType::Lzip|ArcType::Lzma|ArcType::Lzop|ArcType::Lrzip|ArcType::Rzip => r,
		ArcType::SevenZ => {
			match r {
				0   => 0,
				1|2 => 1,
				3|4 => 3,
				5|6 => 5,
				7|8 => 7,
				9   => 9,
				_   => { panic!(); }
			}
		},
		ArcType::Rar => {
			match r {
				0   => 0,
				1|2 => 1,
				3|4 => 2,
				5|6 => 3,
				7|8 => 4,
				9   => 5,
				_   => { panic!(); }
			}
		},
		ArcType::Brotli => {
			match r {
				0|1|2|3|4|5|6 => r,
				7 => 8,
				8 => 10,
				9 => 11,
				_ => { panic!(); }
			}
		},
		ArcType::Zstd => {
			match r {
				0 => 0,
				1 => 1,
				2 => 3,
				3 => 5,
				4 => 8,
				5 => 11,
				6 => 13,
				7 => 15,
				8 => 17,
				9 => 19,
				_ => { panic!(); }
			}
		},
		_ => 0
	};

}