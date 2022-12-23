use crate::errors::Errcode;
use anyhow::{self};
use log::{debug, error};
use nix::unistd::sethostname;

///ランダムなホスト名を生成
pub fn generate_host() -> anyhow::Result<String> {
    let host1 = random_word::gen_len(4).ok_or(Errcode::WordError)?;
    let host2 = random_word::gen_len(4).ok_or(Errcode::WordError)?;
    Ok(format!("{}-{}", host1, host2))
}

///ホスト名をセット
pub fn set_container_hostname(hostname: &String) -> anyhow::Result<()> {
    match sethostname(hostname) {
        Ok(_) => {
            debug!("Container hostname is now {}", hostname);
            Ok(())
        }
        Err(_) => {
            error!("Can not set hostname {}", hostname);
            Err(Errcode::HostnameError(0).into())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_host_success() {
        let host = generate_host();
        println!("{:?}", host);
        assert!(host.unwrap().len() > 3);
    }
}
