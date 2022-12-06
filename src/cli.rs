use clap::Parser;
use log::*;
use simplelog::*;
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(name = "Bowl RS", author = "syuta", version = "v0.1")]

pub struct BowlArg {
    //debug: デバッグ メッセージまたは通常のログを表示するために使用されます。
    #[clap(short, long)]
    debug: Option<bool>,

    //コンテナ内で実行されるコマンド
    #[clap(short, long)]
    pub command: String,

    //コンテナ内でアプリを実行するために作成されるUserID
    #[clap(short, long)]
    pub uid: u32,

    //コンテナ内のroot directoryとして使うdirectory
    #[clap(short, long)]
    pub mount_dir: PathBuf,
}

pub fn parse_args() -> BowlArg {
    let args = BowlArg::parse();

    //set log level
    match args.debug {
        Some(debug) if debug => setting_log(LevelFilter::Debug),
        _ => setting_log(LevelFilter::Info),
    }

    args
}

/// log level setting.
fn setting_log(log_level: LevelFilter) {
    CombinedLogger::init(vec![
        TermLogger::new(
            log_level,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            log_level,
            Config::default(),
            File::create("bowl.log").unwrap(),
        ),
    ])
    .unwrap();
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn parse_success() {
//         let larger = Rectangle { length: 8, width: 7 };
//         let smaller = Rectangle { length : 5, width: 1 };

//         assert!(larger.can_hold(&smaller));
//     }
// }
