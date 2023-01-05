use crate::errors::Errcode;
use crate::host::generate_host;
use crate::ipc::create_sockets;

use std::ffi::CString;
use std::os::unix::io::RawFd;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ContainerOptions {
    ///コンテナ内で実行するプログラムのパス
    pub path: CString,
    ///CLIに渡す引数
    pub args: Vec<CString>,
    //コンテナ内でアプリを実行するために作成されるUserID(0=root)
    pub uid: u32,
    //コンテナ内のroot directoryとして使うdirectory
    pub mount_directory: PathBuf,
    //file descripter
    pub fd: RawFd,
    ///ホスト名
    pub hostname: String,
    //追加パス
    pub add_paths: Vec<(PathBuf, PathBuf)>,
}

impl ContainerOptions {
    pub fn new(
        command: String,
        uid: u32,
        mount_directory: PathBuf,
        add_paths: Vec<(PathBuf, PathBuf)>,
    ) -> Result<(ContainerOptions, (RawFd, RawFd)), Errcode> {
        let _sockets = create_sockets()?;

        let args: Vec<CString> = command
            .split_ascii_whitespace()
            .map(|s| CString::new(s).unwrap())
            .collect();
        let path = args[0].clone();

        let sockets = create_sockets()?;

        Ok((
            ContainerOptions {
                path,
                args,
                uid,
                mount_directory,
                fd: sockets.1,
                hostname: generate_host()?,
                add_paths,
            },
            sockets,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATH: &str = "./test";
    const COMMAND: &str = "bash";

    #[test]
    fn config_new_success() {
        let mut add_paths = vec![];
        let frompath_1 = PathBuf::from("foo");
        let mntpath_1 = PathBuf::from("bar");
        add_paths.push((frompath_1, mntpath_1));
        let frompath_2 = PathBuf::from("buzz");
        let mntpath_2 = PathBuf::from("hoge");
        add_paths.push((frompath_2, mntpath_2));

        let pb = PathBuf::from(PATH);
        let config = ContainerOptions::new(COMMAND.to_string(), 0, pb, add_paths);
        let args = vec![CString::new("bash").unwrap()];
        println!("{:?}", config);
        match config {
            Ok((config, (row_fd1, row_fd2))) => {
                assert_eq!(config.path, CString::new(COMMAND).unwrap());
                assert_eq!(config.args, args);
                assert_eq!(config.uid, 0);
                assert_eq!(config.mount_directory, PathBuf::from(PATH));
                assert_eq!(config.add_paths[0].0, PathBuf::from("foo"));
                assert_eq!(config.add_paths[0].1, PathBuf::from("bar"));
                assert_eq!(config.add_paths[1].0, PathBuf::from("buzz"));
                assert_eq!(config.add_paths[1].1, PathBuf::from("hoge"));
                assert!(row_fd1 > 0);
                assert!(row_fd2 > 0);
            }
            Err(_) => panic!("assert error"),
        }
    }
}
