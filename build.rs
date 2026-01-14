use vergen_gitcl::{Emitter, GitclBuilder};

fn main() {
    let gitcl = GitclBuilder::default()
        .sha(true)
        .build()
        .expect("Failed to build GitclBuilder.");

    Emitter::default()
        .add_instructions(&gitcl)
        .expect("Failed to add instructions.")
        .emit()
        .expect("Failed to emit vergen.");
}
