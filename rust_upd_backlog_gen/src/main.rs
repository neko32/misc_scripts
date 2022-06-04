#![warn(clippy::all, clippy::pedantic)]

use rust_upd_backlog_gen::update_check;
use std::env;

fn main() -> core::result::Result<(), anyhow::Error> {
    let host_name = hostname::get()?;
    let home = env::var("HOME")?;
    let conf_store = format!("{}/resource/rust/rust_upd_backlog_gen/conf.json", home);

    match update_check::run_rustup_chk() {
        Ok((rez,  ver)) if rez => {
            println!("rust version {} @{} is up to date.", ver, host_name.to_string_lossy());
            Ok(())
        },
        Ok((rez,  ver)) if !rez => {
            println!("rust version {} @{} is NOT up to date. Creating JIRA backlog now .. ", ver, host_name.to_string_lossy());
            update_check::create_jira_backlog(&conf_store, &ver)?;
            Ok(())
        },
        Err(e) => Err(e),
        _ => panic!("unexpected issues happened..")
    }
}
