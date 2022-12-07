use crate::errors::Errcode;

use clap::Parser;
use log::*;
use simplelog::*;
use std::fs::File;
use std::path::PathBuf;

use std::os::unix::io::RawFd;

use anyhow::{self};

#[derive(Debug, Parser)]
#[clap(name = "Bowl RS", author = "syuta", version = "v0.1")]
pub struct BowlArg {
    //ログメッセージのレベルを設定するために使用
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
    pub mount_directory: PathBuf,
}

/// parse argument

pub fn parse_args() -> anyhow::Result<BowlArg> {
    let args = BowlArg::parse();

    //set log level
    match args.debug {
        Some(debug) if debug => setting_log(LevelFilter::Debug),
        _ => setting_log(LevelFilter::Info),
    }

    if !args.mount_directory.exists() || !args.mount_directory.is_dir() {
        return Err(Errcode::InvalidArgument("mount_directory").into());
    }

    Ok(args)
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
