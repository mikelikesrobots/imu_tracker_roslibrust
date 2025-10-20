fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p = vec![
        "assets/common_interfaces".into(),
        "assets/rcl_interfaces/builtin_interfaces".into(),
        "../mult_msgs".into(),
    ];

    let (source, dependent_paths) =
        roslibrust::codegen::find_and_generate_ros_messages_without_ros_package_path(p)?;

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("messages.rs");
    std::fs::write(dest_path, source.to_string())?;

    for path in &dependent_paths {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    Ok(())
}
