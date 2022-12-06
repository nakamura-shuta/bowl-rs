mod cli;

use log::{debug, info};

use cli::{parse_args, BowlArg};

fn main() {
    let arg: BowlArg = parse_args();

    info!("info ==== {:?}", arg);
    debug!("debug ===== {:?}", arg);
}
