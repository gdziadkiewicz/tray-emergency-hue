use embed_resource::compile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    compile("resources.rc", embed_resource::NONE).manifest_required()?;
    Ok(())
}