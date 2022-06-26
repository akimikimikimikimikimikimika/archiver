use std::env::current_dir;
use std::fs;
use crate::library::*;
use crate::args::*;

pub fn create(mut d:CreateData) {

	// カレントディレクトリ
	let cd = current_dir().expect("カレントディレクトリが存在しません");

	// アーカイブの種類を判定
	let arc_type = match d.arc_type {
		Some(t) => t,
		None => {
			match guess_type(&d.output,true) {
				Some(t) => t,
				None => ArcType::Zip
			}
		}
	};

	// 入力ファイルの String を Path に変換
	let ip = d.input.iter().map(|i| Path::new(i) ).collect::<Vec<_>>();

	// ファイルが全て存在することを確認
	for (p,i) in izip!(ip.iter(),d.input.iter()) {
		if !p.exists() {
			error(format!("ファイルが存在しません: {}",i));
		}
	}

	// 保存先があることを確認
	let op = Path::new(&d.output).absolute_path();
	match op.parent() {
		Some(p) => {
			if !p.is_dir() {
				error("アーカイブの保存先が存在していません");
			}
		},
		None => {
			error("アーカイブの保存先が正しくありません");
		}
	}

	// パラメータの条件を確認
	rate_conversion(&mut d.rate,&arc_type);

	// 実行モードを選択 (単一ファイル,複数ファイルアーカイブ,空のアーカイブ)
	let filetypes = match ip.len() {
		0 => CreateType::Empty,
		1 => {
			match ip[0] {
				p if p.is_file() && !p.is_symlink() => CreateType::SingleFile,
				p if p.is_dir()  && !p.is_symlink() => CreateType::SingleDir,
				_ => CreateType::Multiple
			}
		},
		_ => CreateType::Multiple
	};

	let tmp = tempdir().expect("作業ディレクトリが作成できませんでした");

	// アーカイブファイルの一時保存先
	let mut archive:OsString = OsString::new();
	// アーカイブを一時保存先に保存して利用するかどうか
	let mut use_temp_position:bool = true;

	// コマンドを用意
	let cl:Vec<Cmd> = match (filetypes,arc_type) {
		(CreateType::Empty,ArcType::Zip) => {
			archive = tmp.join_str("archive.zip");
			let empty_dir = tmp.join_str("_");
			if let Err(_) = fs::create_dir(&empty_dir) { error("アーカイブ作成ができませんでした"); }

			["-r","-d"].into_iter().map(|flag| {
				let mut args = vec![flag];
				if !d.verbose { args.push("-q"); }
				args.extend(["archive.zip","_"]);
				Cmd::new_cwd("zip",vs(args),tmp.path().to_path_buf())
			}).collect()
		},
		(_,ArcType::Zip) => {
			archive = tmp.join_str("archive.zip");

			get_pi(&ip,&d.keep_path).into_iter().map(|(p,i)| {
				let mut c = Cmd::new_cwd("zip",vs(["-r","-x",".*","-x","__MACOSX"]),p);
				if !d.verbose { c.args.push(OsString::from("-q")); }
				c.args.extend([
					OsString::from(format!("-{}",d.rate)),
					OsString::from(&archive),i
				]);
				c
			}).collect()
		},
		(CreateType::Empty,ArcType::SevenZ) => {
			archive = tmp.join_str("archive.7z");
			let empty_dir = tmp.join_str("_");
			if let Err(_) = fs::create_dir(&empty_dir) { error("アーカイブ作成ができませんでした"); }

			[
				vs(["a","-ba","-t7z","archive.7z","_"]),
				vs(["d","-ba","archive.7z","_"])
			].into_iter().map(|args| {
				let mut c = Cmd::new_cwd("7z",args,tmp.path().to_path_buf());
				c.inherit_output = d.verbose;
				c
			}).collect::<Vec<Cmd>>()
		},
		(_,ArcType::SevenZ) => {
			archive = tmp.join_str("archive.7z");

			get_pi(&ip,&d.keep_path).into_iter().map(|(p,i)| {
				let mut c = Cmd::new_cwd("7z",vs(["a","-ba","-t7z","-xr!.*"]),p);
				c.args.extend([
					OsString::from(format!("-mx={}",d.rate)),
					OsString::from(&archive),i
				]);
				c.inherit_output = d.verbose;
				c
			}).collect()
		},
		(CreateType::Empty,ArcType::Rar) => {
			error("空の RAR アーカイブは作成できません");
			panic!();
		},
		(_,ArcType::Rar) => {
			archive = tmp.join_str("archive.rar");

			get_pi(&ip,&d.keep_path).into_iter().map(|(p,i)| {
				let mut c = Cmd::new_cwd("rar",vs(["a","-r"]),p);
				if !d.verbose { c.args.push(OsString::from("-inul")); }
				c.args.extend([
					OsString::from(format!("-m{}",d.rate)),
					OsString::from(&archive),i
				]);
				c
			}).collect()
		},
		(CreateType::Empty,ArcType::Aar) => {
			archive = tmp.join_str("archive.aar");

			let mut c = Cmd::new_cwd(
				"aa",
				vs(["archive","-d",".","-include-path","_","-o","archive.aar"]),
				tmp.path().to_path_buf()
			);
			c.args[2] = OsString::from(tmp.path());
			if d.verbose { c.args.push(OsString::from("-v")); }
			vec![c]
		},
		(CreateType::SingleDir,ArcType::Aar) => {
			archive = tmp.join_str("archive.aar");

			let mut c = Cmd::new("aa",vs(["archive","-d",&d.input[0],"-o"]));
			if d.verbose { c.args.push(OsString::from("-v")); }
			c.args.push(OsString::from(&archive));
			vec![c]
		},
		(_,ArcType::Aar) => {
			archive = tmp.join_str("archive.aar");

			let mut c = Cmd::new("aa",vs(["archive","-d",".","-o"]));
			c.args[2] = OsString::from(&cd);
			if d.verbose { c.args.push(OsString::from("-v")); }
			c.args.push(OsString::from(&archive));
			for i in d.input.iter() {
				c.args.extend(vs(["-include-path",i]));
			}
			vec![c]
		},
		(CreateType::Empty,ArcType::Wim) => {
			archive = tmp.join_str("image.wim");
			let empty_dir = tmp.join_str("_");
			if let Err(_) = fs::create_dir(&empty_dir) { error("アーカイブ作成ができませんでした"); }

			vec![
				Cmd::new_cwd(
					"wimcapture",
					vs(["_","image.wim"]),
					tmp.path().to_path_buf()
				)
			]
		},
		(CreateType::SingleDir,ArcType::Wim) => {
			archive = tmp.join_str("image.wim");

			let mut c = Cmd::new("wimcapture",vs([&d.input[0]]));
			c.args.push(OsString::from(&archive));
			vec![c]
		},
		(CreateType::Empty,ArcType::Dmg) => {
			use_temp_position = false;
			let empty_dir = tmp.join_str(d.image_name);
			if let Err(_) = fs::create_dir(&empty_dir) { error("アーカイブ作成ができませんでした"); }

			let mut c = Cmd::new(
				"hdiutil",
				vs([
					"create","-nospotlight",
					"-layout","GPTSPUD",
					"-fs","Case-sensitive APFS",
					"-type","UDZO",
					"-srcfolder","."
				])
			);
			c.args[10] = empty_dir;
			if !d.verbose { c.args.push(OsString::from("-quiet")); }
			c.args.push(OsString::from(&d.output));
			vec![c]
		},
		(CreateType::SingleDir,ArcType::Dmg) => {
			archive = tmp.join_str("image.dmg");

			let mut c = Cmd::new(
				"hdiutil",
				vs([
					"create","-quiet","-nospotlight",
					"-layout","GPTSPUD",
					"-fs","Case-sensitive APFS",
					"-type","UDZO",
					"-srcfolder",&d.input[0]
				])
			);
			if !d.verbose { c.args.push(OsString::from("-quiet")); }
			c.args.push(OsString::from(&archive));
			vec![c]
		},
		(CreateType::Empty,ArcType::Iso) => {
			use_temp_position = false;
			let empty_dir = tmp.join_str(d.image_name);
			if let Err(_) = fs::create_dir(&empty_dir) { error("アーカイブ作成ができませんでした"); }

			let mut c = Cmd::new(
				"hdiutil",
				vs(["makehybrid","-iso","-joliet","-o",&d.output])
			);
			if !d.verbose { c.args.push(OsString::from("-quiet")); }
			c.args.push(OsString::from(&empty_dir));
			vec![c]
		},
		(CreateType::SingleDir,ArcType::Iso) => {
			archive = tmp.join_str("image.iso");

			let mut c = Cmd::new(
				"hdiutil",
				vs(["makehybrid","-iso","-joliet","-o",".",&d.input[0]])
			);
			c.args[5] = OsString::from(&archive);
			if !d.verbose { c.args.push(OsString::from("-quiet")); }
			vec![c]
		},
		(CreateType::Empty,ArcType::Lha) => {
			archive = tmp.join_str("archive.lhz");
			let empty_dir = tmp.join_str("_");
			if let Err(_) = fs::create_dir(&empty_dir) { error("アーカイブ作成ができませんでした"); }

			let mut c = Cmd::new_cwd(
				"lha",
				vs(["-a","archive.lzh","-x=_"]),
				tmp.path().to_path_buf()
			);
			if !d.verbose { c.args.push(OsString::from("-q")); }
			c.args.push(OsString::from("_"));
			vec![c]
		},
		(_,ArcType::Lha) => {
			archive = tmp.join_str("archive.lhz");

			get_pi(&ip,&d.keep_path).into_iter().map(|(p,i)| {
				let mut c = Cmd::new_cwd("lha",vs(["-a"]),p);
				if !d.verbose { c.args.push(OsString::from("-q")); }
				c.args.extend([OsString::from(&archive),i]);
				c
			}).collect()
		},
		(CreateType::Empty,ArcType::Zpaq) => {
			error("空の ZPAQ アーカイブは作成できません");
			panic!();
		},
		(_,ArcType::Zpaq) => {
			archive = tmp.join_str("archive.zpaq");

			get_pi(&ip,&d.keep_path).into_iter().map(|(p,i)| {
				let mut c = Cmd::new_cwd("zpaq",vs(["a"]),p);
				c.args.extend([OsString::from(&archive),i]);
				c
			}).collect()
		},
		(_,ArcType::Wim)|(_,ArcType::Dmg)|(_,ArcType::Iso) => {
			error("WIM/DMG/ISO は単一のフォルダから作成することができます。");
			panic!();
		},
		(CreateType::Empty,ArcType::Cpio) => {
			archive = tmp.join_str("archive.cpio");

			let mut c = Cmd::new_cwd(
				"cpio",
				vs(["--create","-O","archive.cpio"]),
				tmp.path().to_path_buf()
			);
			if !d.verbose { c.args.push(OsString::from("--quiet")); }
			vec![c]
		},
		(CreateType::SingleFile,ArcType::Cpio)|(CreateType::SingleDir,ArcType::Cpio) => {
			archive = tmp.join_str("archive.cpio");

			let v = get_pi(&ip,&d.keep_path);
			let mut c = Cmd::new_cwd_stdin(
				"cpio",
				vs(["--create","--null","-O","archive.cpio"]),
				v[0].0.as_path().to_path_buf(),
				v[0].1.to_str().unwrap().to_string()
			);
			if !d.verbose { c.args.push(OsString::from("--quiet")); }
			vec![c]
		},
		(CreateType::Multiple,ArcType::Cpio) => {
			archive = tmp.join_str("archive.cpio");

			let mut c = Cmd::new_cwd_stdin(
				"cpio",
				vs(["--create","--null","-O"]),
				cd,
				d.input.join("\0")
			);
			if !d.verbose { c.args.push(OsString::from("--quiet")); }
			c.args.push(OsString::from(&archive));
			vec![c]
		},
		// tar と圧縮系をここに集約
		(ct,at) => tar_or_compress(ct,at,&ip,&d.rate,&d.keep_path,&mut archive,&tmp)
	};

	// コマンドを実行
	if several_cmd(cl) {
		if use_temp_position {
			if !Path::new(&archive).is_file() {
				error("アーカイブは作成されていません。");
			}
			if let Err(_) = fs::hard_link(&archive,&d.output) {
				if let Err(_) = fs::copy(&archive,&d.output) {
					error("アーカイブの保存に失敗しました。");
				}
			}
		}
	}
	else { error("アーカイブの作成に失敗しました"); }

	tmp.close().expect("作業ディレクトリが完全には削除されませんでした");

}

/// (カレントディレクトリ,入力ファイル) のペアに変換
fn get_pi(i:&Vec<&Path>,keep_path:&bool) -> Vec<(PathBuf,OsString)> {

	match *keep_path {
		true => {
			i.iter().map( |p| {
				(
					Path::new(".").absolute_path(),
					p.to_path_buf().into_os_string()
				)
			}).collect()
		},
		false => {
			i.iter().map( |p| {
				let dir = p.parent();
				let base = p.file_name();
				match (dir,base) {
					(Some(d),Some(b)) => (d.absolute_path(),b.to_os_string()),
					_ => {
						error("パスが存在しません");
						panic!();
					}
				}
			}).collect()
		}
	}
}

/// tar アーカイブ / 圧縮
fn tar_or_compress(
	ct:CreateType,at:ArcType,
	i:&Vec<&Path>,rate:&u8,keep_path:&bool,archive:&mut OsString,tmp:&TempDir
) -> Vec<Cmd> {

	// 単一ファイルの圧縮の場合とそうでない場合に分離
	let compress = match (&ct,&at) {
		(_,ArcType::Tar) => false,
		(CreateType::SingleFile,_) => true,
		_ => false
	};

	if compress {
		let v = get_pi(i,keep_path);
		let src_name = v[0].1.as_os_str().to_str().unwrap();
		let dst_name = format!("{}.{}",&src_name,compress_ext(&at));
		let src = i[0].to_path_buf();
		let src_tmp = tmp.join_str(&src_name);
		let dst_tmp = tmp.join_str(&dst_name);
		*archive = dst_tmp;
		if let Err(_) = fs::hard_link(&src,&src_tmp) {
			if let Err(_) = fs::copy(&src,&src_tmp) {
				error("圧縮が開始できませんでした。");
			}
		}

		// 圧縮
		let mut c = match &at {
			ArcType::Compress => Cmd::new("compress",vs(["-f",                              ])),
			ArcType::Gzip     => Cmd::new("gzip"    ,vs(["-f","-k"     ,&format!("-{}",rate)])),
			ArcType::Bzip2    => Cmd::new("bzip2"   ,vs(["-z","-f","-k",&format!("-{}",rate)])),
			ArcType::Xz       => Cmd::new("xz"      ,vs(["-z","-f","-k",&format!("-{}",rate)])),
			ArcType::Lzip     => Cmd::new("lzip"    ,vs(["-k",          &format!("-{}",rate)])),
			ArcType::Lzma     => Cmd::new("lzma"    ,vs(["-z","-f","-k",&format!("-{}",rate)])),
			ArcType::Lz4      => Cmd::new("lz4"     ,vs(["-z","-q",     &format!("-{}",rate)])),
			ArcType::Lzop     => Cmd::new("lzop"    ,vs([               &format!("-{}",rate)])),
			ArcType::Lrzip    => Cmd::new("lrzip"   ,vs(["-q",     "-L",&format!( "{}",rate)])),
			ArcType::Rzip     => Cmd::new("rzip"    ,vs(["-k",          &format!("-{}",rate)])),
			ArcType::Zstd     => Cmd::new("zstd"    ,vs(["-z","-q",     &format!("-{}",rate)])),
			ArcType::Brotli   => Cmd::new("brotli"  ,vs([          "-q",&format!( "{}",rate)])),
			ArcType::Lzfse    => Cmd::new("aa"      ,vs(["archive","-o",&dst_name,"-i"      ])),
			_ => { panic!(); }
		};
		c.args.push(OsString::from(src_name));
		c.cwd = tmp.path().to_path_buf();
		vec![c]
	}
	else {
		*archive = tmp.join_str("archive.tar");

		// tar アーカイブの部分
		let mut l = match &ct {
			CreateType::Empty => {
				vec![
					Cmd::new_cwd(
						"bsdtar",
						vs(["-c","-f","archive.tar","-T","/dev/null"]),
						tmp.path().to_path_buf()
					)
				]
			},
			_ => {
				get_pi(i,keep_path).into_iter().enumerate().map(|(index,(p,i))| {
					let mut c = Cmd::new_cwd_env(
						"bsdtar",
						match index {
							0 => vs(["-c","-f"]),
							_ => vs(["-r","-f"])
						},
						p,
						[("COPYFILE_DISABLE","1")]
					);
					c.args.extend([OsString::from(&archive),i]);
					c
				}).collect::<Vec<Cmd>>()
			}
		};

		// 圧縮の部分
		match &at {
			ArcType::Tar => {},
			_ => {
				let mut c = match &at {
					ArcType::Compress => Cmd::new("compress",vs(["-f",                                  ])),
					ArcType::Gzip     => Cmd::new("gzip"    ,vs([                 &format!("-{}",rate)  ])),
					ArcType::Bzip2    => Cmd::new("bzip2"   ,vs(["-z",            &format!("-{}",rate)  ])),
					ArcType::Xz       => Cmd::new("xz"      ,vs(["-z",            &format!("-{}",rate)  ])),
					ArcType::Lzip     => Cmd::new("lzip"    ,vs([                 &format!("-{}",rate)  ])),
					ArcType::Lzma     => Cmd::new("lzma"    ,vs(["-z",            &format!("-{}",rate)  ])),
					ArcType::Lz4      => Cmd::new("lz4"     ,vs(["-z","-q","--rm",&format!("-{}",rate)  ])),
					ArcType::Lzop     => Cmd::new("lzop"    ,vs(["-U",            &format!("-{}",rate)  ])),
					ArcType::Lrzip    => Cmd::new("lrzip"   ,vs(["-q","-D",  "-L",&format!( "{}",rate)  ])),
					ArcType::Rzip     => Cmd::new("rzip"    ,vs(["-U",            &format!("-{}",rate)  ])),
					ArcType::Zstd     => Cmd::new("zstd"    ,vs(["-z","-q","--rm",&format!("-{}",rate)  ])),
					ArcType::Brotli   => Cmd::new("brotli"  ,vs(["--rm",     "-q",&format!( "{}",rate)  ])),
					ArcType::Lzfse    => Cmd::new("aa"      ,vs(["archive","-o","archive.tar.lzfse","-i"])),
					_ => { panic!(); }
				};
				c.args.extend(vs(["archive.tar"]));
				*archive = tmp.join_str(
					format!("archive.tar.{}",compress_ext(&at))
				);
				c.cwd = tmp.path().to_path_buf();
				l.push(c);
			}
		}

		return l;
	}

}

enum CreateType {
	SingleFile,
	SingleDir,
	Multiple,
	Empty
}