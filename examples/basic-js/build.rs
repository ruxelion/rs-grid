fn main() {
    // Concatenate theme CSS files from example-common/themes/
    // into a single rs-grid-theme.css for the browser.
    let manifest = env!("CARGO_MANIFEST_DIR");
    let themes_dir = format!("{manifest}/../example-common/themes");
    let dst = format!("{manifest}/rs-grid-theme.css");

    let parts = ["base", "light", "dark", "material", "material-dark"];
    let mut css = String::new();
    for name in parts {
        let path = format!("{themes_dir}/{name}.css");
        css.push_str(
            &std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{path}: {e}")),
        );
        css.push('\n');
        println!("cargo:rerun-if-changed={path}");
    }
    std::fs::write(&dst, css).expect("write theme CSS");
}
