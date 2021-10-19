use serde_generate::{dart, test_utils, CodeGeneratorConfig, Encoding, SourceInstaller};
use std::{io::Result, path::PathBuf, process::Command};
use tempfile::tempdir;

fn install_test_dependency(path: PathBuf) -> Result<()> {
    Command::new("dart")
        .current_dir(path)
        .args(["pub", "add", "-d", "test"])
        .status()?;

    Ok(())
}
#[test]
fn test_dart_runtime() {
    let source_path = tempdir().unwrap().path().join("dart_project");
    // let source_path = std::path::Path::new("../../dart_project");
    let registry = test_utils::get_registry().unwrap();

    let config = CodeGeneratorConfig::new("example".to_string())
        .with_encodings(vec![Encoding::Bcs, Encoding::Bincode])
        .with_c_style_enums(true);

    let installer = dart::Installer::new(source_path.to_path_buf());
    installer.install_module(&config, &registry).unwrap();
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();
    installer.install_bcs_runtime().unwrap();

    let _result = install_test_dependency(source_path.to_path_buf());

    let dart_test = Command::new("dart")
        .current_dir(source_path.to_path_buf())
        .args(["test"])
        .status()
        .unwrap();

    assert!(dart_test.success());
}
