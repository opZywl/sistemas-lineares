fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=../../assets/app.ico");
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        if let Err(error) = embed_resource::compile_for_everything("app.rc", embed_resource::NONE)
            .manifest_required()
        {
            panic!("{error}");
        }
    }
}
