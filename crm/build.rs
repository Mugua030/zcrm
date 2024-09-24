use anyhow::Result;
use proto_builder_trait::tonic::BuilderAttributes;
use std::fs;

fn main() -> Result<()> {
    fs::create_dir_all("src/pb")?;

    tonic_build::configure()
        .out_dir("src/pb")
        .with_derive_builder(&["WelcomeRequest"], None)
        .with_field_attributes(
            &["WelcomeRequest.content_ids"],
            &[r#"#[builder(setter(each(name="content_id", into)))]"#],
        )
        .compile(&["../protos/crm/crm.proto"], &["../protos"])?;

    Ok(())
}
