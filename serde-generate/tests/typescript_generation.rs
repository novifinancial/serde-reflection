// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_generate::{test_utils, typescript, CodeGeneratorConfig, Encoding, SourceInstaller};
use std::{
    fs::File,
    io::{Result, Write},
    process::Command,
};
use tempfile::{tempdir, TempDir};

fn write_package_tsconfig_json_for_test_build(path: std::path::PathBuf) -> Result<()> {
    let mut package_json = std::fs::File::create(path.join("package.json"))?;
    writeln!(
        package_json,
        r#"
{{
  "name": "tmpCode",
  "version": "1.0.0",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {{
    "build": "tsc"
  }},
  "dependencies": {{
  }},
  "devDependencies": {{
    "@types/node": "12.12.2",
    "typescript": "^3.9.6"
  }}
}}
"#
    )?;
    let mut tsconfig_json = std::fs::File::create(path.join("tsconfig.json"))?;
    writeln!(
        tsconfig_json,
        r#"
{{
  "compilerOptions": {{
    "target": "es6",
    "module": "commonjs",
    "declaration": true,
    "outDir": "./dist",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "lib": [
      "es6",
      "esnext.BigInt",
      "dom"
    ]

  }},
  "include": ["testing/*.ts"]
}}
"#
    )?;
    Ok(())
}

fn test_that_ts_code_compiles_with_config(
    config: &CodeGeneratorConfig,
) -> (TempDir, std::path::PathBuf) {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();
    make_output_file(&dir);
    let source_path = dir.path().join("testing").join("test.ts");
    let mut source = File::create(&source_path).unwrap();

    let generator = typescript::CodeGenerator::new(&config);
    generator.output(&mut source, &registry).unwrap();
    let _result = write_package_tsconfig_json_for_test_build(dir.path().to_path_buf());

    let installer = typescript::Installer::new(dir.path().to_path_buf());
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();
    installer.install_bcs_runtime().unwrap();

    let npm_status = Command::new("npm")
        .arg("install")
        .current_dir(dir.path())
        .status()
        .unwrap();
    assert!(npm_status.success());

    let status = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir(dir.path())
        .status()
        .unwrap();
    assert!(status.success());

    let path = dir.path().join("testing");
    (dir, path)
}

fn make_output_file(dir: &TempDir) {
    std::fs::create_dir_all(dir.path().join("testing")).unwrap_or(());
}

#[test]
fn test_that_ts_code_compiles() {
    let config = CodeGeneratorConfig::new("testing".to_string());
    test_that_ts_code_compiles_with_config(&config);
}

#[test]
fn test_that_ts_code_compiles_with_bcs() {
    let config =
        CodeGeneratorConfig::new("testing".to_string()).with_encodings(vec![Encoding::Bcs]);
    test_that_ts_code_compiles_with_config(&config);
}

#[test]
fn test_that_ts_code_compiles_with_comments() {
    let comments = vec![(vec!["SerdeData".to_string()], "Some\ncomments".to_string())]
        .into_iter()
        .collect();
    let config = CodeGeneratorConfig::new("testing".to_string()).with_comments(comments);

    let (_dir, path) = test_that_ts_code_compiles_with_config(&config);

    // Comment was correctly generated.
    let content = std::fs::read_to_string(path.join("test.ts")).unwrap();
    assert!(content.contains(
        r#"
/**
 * Some
 * comments
 */
"#
    ));
}
