use anyhow::Result;
use std::fs;

fn main() -> Result<()> {
    fs::create_dir_all("src/pb")?;
    tonic_build::configure()
        .out_dir("src/pb")
        .compile(&["../protos/notify-svc/notify.proto"], &["../protos"])
        .unwrap();

    Ok(())
}
