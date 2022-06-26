use std::env::current_dir;
use std::fs;
use crate::library::*;
use crate::args::*;

pub fn extract(d:ExtractData) {

	// アーカイブの種類を判定
	let arc_type = match d.arc_type {
		Some(t) => t,
		None => {
			match guess_type(&d.input,true) {
				Some(t) => t,
				None => ArcType::Tar
			}
		}
	};

	// アーカイブが存在することを確認
	if !Path::new(&d.input).is_file() {
		error("ファイルが存在しません");
	}

	// 保存先の存在を確認
	let output = check_output(&d.input,&d.output,&arc_type);

	let tmp = tempdir().expect("作業ディレクトリが作成できませんでした");

	// コマンドを実行
	match arc_type {
		ArcType::Zip => {
			let mut c = Cmd::new("unzip",vs(["-q","-d",&output,&d.input]));
			for t in d.target.iter() {
				c.args.push(OsString::from(&t));
			}
			if !several_cmd(vec![c]) { error("展開に失敗しました"); }
		},
		ArcType::SevenZ => {
			let mut c = Cmd::new("7z",vs(["x","-ba",&d.input,&format!("-o{}",output)]));
			for t in d.target.iter() {
				c.args.push(OsString::from(&t));
			}
			if !several_cmd(vec![c]) { error("展開に失敗しました"); }
		},
		ArcType::Tar|ArcType::Cpio => {
			let mut c = Cmd::new("bsdtar",vs(["-x","-f",&d.input,"-C",&output]));
			for t in d.target.iter() {
				c.args.push(OsString::from(&t));
			}
			if !several_cmd(vec![c]) { error("展開に失敗しました"); }
		},
		ArcType::Rar => {
			let mut c = Cmd::new("rar",vs(["x","-inul",&d.input,&output]));
			for t in d.target.iter() {
				c.args.push(OsString::from(&t));
			}
			if !several_cmd(vec![c]) { error("展開に失敗しました"); }
		},
		ArcType::Aar => {
			let mut c = Cmd::new("aa",vs(["extract","-i",&d.input,"-d",&output]));
			for t in d.target.iter() {
				c.args.extend(vs(["-include-path",&t]));
			}
			if !several_cmd(vec![c]) { error("展開に失敗しました"); }
		},
		ArcType::Wim => {
			if d.target.len()>0 {
				eprintln!("WIM では --target フラグは無視されます");
			}
			let c = Cmd::new("wimapply",vs([&d.input,&output]));
			if !several_cmd(vec![c]) { error("展開に失敗しました"); }
		},
		ArcType::Zpaq => {
			let mut c = Cmd::new("zpaq",vs(["x",&d.input,"-to",&output]));
			for t in d.target.iter() {
				c.args.push(OsString::from(&t));
			}
			if !several_cmd(vec![c]) { error("展開に失敗しました"); }
		},
		ArcType::Lha => {
			let mut c = Cmd::new("lha",vs(["-x","-q",&d.input,"-w",&output]));
			for t in d.target.iter() {
				c.args.push(OsString::from(&t));
			}
			if !several_cmd(vec![c]) { error("展開に失敗しました"); }
		},
		ArcType::Dmg|ArcType::Iso => {
			error("このファイルは展開に対応していません");
		},
		_ => {
			let src_name = OsString::from(
				format!("file.{}",compress_ext(&arc_type))
			);
			let src_tmp = tmp.path().join(&src_name);
			let dst_tmp = tmp.path().join("file");
			if let Err(_) = fs::hard_link(&d.input,&src_tmp) {
				if let Err(_) = fs::copy(&d.input,&src_tmp) {
					error("解凍が開始できませんでした。");
				}
			}

			let mut c = match &arc_type {
				ArcType::Compress => Cmd::new("uncompress",vs([     "-f"])),
				ArcType::Gzip     => Cmd::new("gzip"      ,vs(["-d","-f"])),
				ArcType::Bzip2    => Cmd::new("bzip2"     ,vs(["-d","-f"])),
				ArcType::Xz       => Cmd::new("xz"        ,vs(["-d","-f"])),
				ArcType::Lzip     => Cmd::new("lzip"      ,vs(["-d",    ])),
				ArcType::Lzma     => Cmd::new("lzma"      ,vs(["-d","-f"])),
				ArcType::Lz4      => Cmd::new("lz4"       ,vs(["-d","-q"])),
				ArcType::Lzop     => Cmd::new("lzop"      ,vs(["-d"     ])),
				ArcType::Lrzip    => Cmd::new("lrzip"     ,vs(["-d","-q"])),
				ArcType::Rzip     => Cmd::new("rzip"      ,vs(["-d"     ])),
				ArcType::Zstd     => Cmd::new("zstd"      ,vs(["-d","-q"])),
				ArcType::Brotli   => Cmd::new("brotli"    ,vs(["-d","-q"])),
				ArcType::Lzfse    => Cmd::new("aa"        ,vs(["extract","-o","file","-i","file.lzfse"])),
				_ => { panic!(); }
			};
			c.args.push(src_name);
			c.cwd = tmp.path().to_path_buf();

			if several_cmd(vec![c]) {
				if !dst_tmp.is_file() {
					error("解凍に失敗しました");
				}
				if let Err(_) = fs::rename(&dst_tmp,&output) {
					if let Err(_) = fs::copy(&dst_tmp,&output) {
						error("解凍ファイルの保存に失敗しました。");
					}
				}
			}
			else { error("解凍に失敗しました"); }
		}
	};

	tmp.close().expect("作業ディレクトリが完全には削除されませんでした");

}

fn check_output(input:&String,output:&Option<String>,arc_type:&ArcType) -> String {
	match (output,arc_type) {
		(oo,ArcType::Zip)|(oo,ArcType::SevenZ)|(oo,ArcType::Tar)|(oo,ArcType::Cpio)|(oo,ArcType::Rar)|(oo,ArcType::Aar)|(oo,ArcType::Wim)|(oo,ArcType::Zpaq)|(oo,ArcType::Lha) => {
			match oo {
				Some(o) => {
					if !Path::new(o).is_dir() {
						error("保存先のディレクトリが存在しません");
					}
					o.to_string()
				},
				None => {
					let cd = current_dir().expect("カレントディレクトリが存在しません");
					cd.to_str().expect("カレントディレクトリに展開できません。パスに非対応の文字が含まれています").to_string()
				}
			}
		},
		(_,ArcType::Dmg)|(_,ArcType::Iso) => {
			error("ディスクイメージには対応していません");
			panic!();
		},
		(oo,at) => {
			match oo {
				Some(o) => {
					match Path::new(o).parent() {
						Some(p) => {
							if !p.is_dir() {
								error("保存先が存在しません");
							}
							o.to_string()
						}
						None => {
							error("保存先が正しくありません");
							panic!();
						}
					}
				},
				None => compress_remove_ext(input,at)
			}
		}
	}
}

fn compress_remove_ext(input:&String,arc_type:&ArcType) -> String {
	macro_rules! replace_suffix {
		($suffix:expr,$repl:expr) => {
			if let Some(s) = input.strip_suffix($suffix) { return s.to_string()+&$repl; }
		};
	}
	match arc_type {
		ArcType::Compress => {
			replace_suffix!(".Z","");
			replace_suffix!(".z","");
		},
		ArcType::Gzip => {
			replace_suffix!(".gz","");
			replace_suffix!(".tgz",".tar");
		},
		ArcType::Bzip2 => {
			replace_suffix!(".bz2","");
			replace_suffix!(".bz","");
			replace_suffix!(".tbz2",".tar");
			replace_suffix!(".tbz",".tar");
		},
		ArcType::Xz => {
			replace_suffix!(".xz","");
			replace_suffix!(".txz",".tar");
		},
		ArcType::Lzip => {
			replace_suffix!(".lz","");
			replace_suffix!(".tlz",".tar");
		},
		ArcType::Lzma => {
			replace_suffix!(".lzma","");
		},
		ArcType::Lz4 => {
			replace_suffix!(".lz4","");
		},
		ArcType::Lzop => {
			replace_suffix!(".lzo","");
		},
		ArcType::Lrzip => {
			replace_suffix!(".lrz","");
		},
		ArcType::Rzip => {
			replace_suffix!(".rz","");
		},
		ArcType::Zstd => {
			replace_suffix!(".zst","");
			replace_suffix!(".tzst",".tar");
		},
		ArcType::Brotli => {
			replace_suffix!(".br","");
		},
		ArcType::Lzfse => {
			replace_suffix!(".lzfse","");
		},
		_ => {}
	}
	return input.clone()+".out";
}