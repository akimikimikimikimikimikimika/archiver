pub use clap::{Parser,Args,Subcommand,ArgEnum};

#[derive(Parser)]
/// アーカイブコマンドのラッパー
pub struct Arguments {
	#[clap(subcommand)]
	pub command: ArcCmd
}

#[derive(Subcommand)]
pub enum ArcCmd {
	/// 新しいアーカイブを作成します。単一ファイルの圧縮やディスクイメージの作成にも対応しています。
	#[clap(name="create",aliases=&["c","archive"])]
	Create(CreateData),
	/// アーカイブにファイルを追加します。
	Append,
	/// アーカイブからファイルを削除します。
	Remove,
	/// アーカイブ内のファイル名を変更します。
	Rename,
	/// アーカイブを展開します。
	Extract(ExtractData),
	/// アーカイブの内容を表示します。
	List(ListData),
	/// このコマンドで使用するパッケージを全てインストールします
	Setup,
	/// ヘルプを表示します。
	Help
}

#[derive(Args)]
pub struct CreateData {
	/// アーカイブに追加するファイルを指定します。指定しない場合は、空のアーカイブが作成されることがあります。
	pub input: Vec<String>,
	#[clap(short,long)]
	/// 出力先となるアーカイブを指定します。
	pub output: String,
	#[clap(short='t',long="type",arg_enum)]
	/// アーカイブの種類を変更します。標準では出力ファイルの拡張子から判定します。圧縮系のフォーマットは複数の入力ファイルに対して自動的に tar アーカイブにした上で圧縮します。
	pub arc_type: Option<ArcType>,
	#[clap(short,long,default_value_t=6)]
	/// 圧縮を伴うアーカイブにおいて圧縮率を指定します。
	pub rate: u8,
	/// 進行状況などを出力します
	#[clap(short,long)]
	pub verbose: bool,
	#[clap(long="image-name",default_value_t=String::from("Untitled"))]
	/// DMG,ISO の場合にディスクの名前を指定します。フォルダから作成する場合は無視されます。
	pub image_name: String,
	#[clap(long="keep-path")]
	/// 追加するファイルのパスをアーカイブの階層構造に使用します。指定しない場合は追加するファイルをルート階層に配置します。
	pub keep_path: bool
}

#[derive(Args)]
pub struct ListData {
	/// リスト表示するアーカイブを指定します。
	pub input: String,
	#[clap(short='t',long="type",arg_enum)]
	/// アーカイブの種類を変更します。標準ではアーカイブの拡張子から判定します。
	pub arc_type: Option<ArcType>,
}

#[derive(Args)]
pub struct ExtractData {
	/// 展開するアーカイブファイルを指定します。
	pub input: String,
	#[clap(short,long)]
	/// アーカイブの展開先となるディレクトリを指定します。或いは、解凍した圧縮ファイルの保存先を指定します。指定しない場合は現在のディレクトリに展開/解凍されます。
	pub output: Option<String>,
	/// 展開対象のファイルを指定します。
	#[clap(short,long)]
	pub target: Vec<String>,
	#[clap(short='t',long="type",arg_enum)]
	/// アーカイブの種類を変更します。標準ではアーカイブファイルの拡張子から判定します。
	pub arc_type: Option<ArcType>,
}

#[derive(ArgEnum,Clone)]
pub enum ArcType {
	#[clap(name="zip")]
	Zip,
	#[clap(name="7z",aliases=["sevenz","seven-zip"])]
	SevenZ,
	#[clap(name="tar")]
	Tar,
	#[clap(name="cpio")]
	Cpio,
	#[clap(name="rar")]
	Rar,
	#[clap(name="aar",aliases=["apple-archive"])]
	Aar,
	#[clap(name="wim",aliases=["windows-image"])]
	Wim,
	#[clap(name="dmg")]
	Dmg,
	#[clap(name="iso")]
	Iso,
	#[clap(name="zpaq")]
	Zpaq,
	#[clap(name="lha",aliases=["lhz"])]
	Lha,
	#[clap(name="compress",aliases=["z","Z","tar.Z"])]
	Compress,
	#[clap(name="gzip",aliases=["gz","tgz","tar.gz","gnuzip"])]
	Gzip,
	#[clap(name="bzip2",aliases=["bz2","tbz2","tar.bz2","bz","tbz"])]
	Bzip2,
	#[clap(name="xz",aliases=["txz","tar.xz"])]
	Xz,
	#[clap(name="lzip",aliases=["lz","tlz","tar.lz"])]
	Lzip,
	#[clap(name="lzma",aliases=["lzma","tar.lzma"])]
	Lzma,
	#[clap(name="lz4",aliases=["tar.lz4"])]
	Lz4,
	#[clap(name="lzop",aliases=["lzo","tar.lzma"])]
	Lzop,
	#[clap(name="lrzip",aliases=["lrz","tar.lrz"])]
	Lrzip,
	#[clap(name="rzip",aliases=["rz","tar.rz"])]
	Rzip,
	#[clap(name="zstd",aliases=["zst","zstandard","tar.zst","tzst"])]
	Zstd,
	#[clap(name="brotli",aliases=["br","tar.br"])]
	Brotli,
	#[clap(name="lzfse",aliases=["tar.lzfse"])]
	Lzfse,
}