fn main() {
    // Copy the canonical theme CSS from example-common.
    let manifest = env!("CARGO_MANIFEST_DIR");
    let src =
        format!("{manifest}/../example-common/rs-grid-theme.css");
    let dst = format!("{manifest}/rs-grid-theme.css");
    std::fs::copy(&src, &dst).expect("copy theme CSS");
    println!("cargo:rerun-if-changed={src}");
}
