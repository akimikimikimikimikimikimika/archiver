use crate::library::*;
use crate::args::*;

pub fn list(d:ListData) {

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

	// コマンドを用意
	let c = match arc_type {
		ArcType::Zip    => Cmd::new("zipinfo",vs(["-1",&d.input])),
		ArcType::Tar|ArcType::Lha => {
			Cmd::new("bsdtar",vs(["-t","-f",&d.input]))
		},
		ArcType::SevenZ => Cmd::new("7z"     ,vs(["l","-ba",&d.input])),
		ArcType::Rar    => Cmd::new("rar"    ,vs(["lb",&d.input])),
		ArcType::Cpio   => Cmd::new("cpio"   ,vs(["-t","-I",&d.input])),
		ArcType::Aar    => Cmd::new("aa"     ,vs(["list","-i",&d.input])),
		ArcType::Wim    => Cmd::new("wimdir" ,vs([&d.input])),
		ArcType::Zpaq   => Cmd::new("zpaq"   ,vs(["l",&d.input])),
		_ => {
			error("このファイルは内容の表示に対応していません");
			panic!();
		}
	};

	// コマンドを実行
	if !several_cmd(vec![c]) { error("内容の表示に失敗しました"); }

}