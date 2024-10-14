use anyhow::Result;
use vergen_git2::{CargoBuilder, Emitter, Git2Builder};

fn main() -> Result<()> {
    let cargo = CargoBuilder::default().target_triple(true).build()?;
    let git2 = Git2Builder::default().sha(true).build()?;

    Emitter::default()
        .add_instructions(&cargo)?
        .add_instructions(&git2)?
        .emit()
}
