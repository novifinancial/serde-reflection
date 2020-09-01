// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_generate::{ts, test_utils, CodeGeneratorConfig, Encoding, SourceInstaller};
use std::process::Command;
use tempfile::{tempdir, TempDir};
use std::io::{Write, Result};


fn write_package_tsconfig_json_for_test_build(path: std::path::PathBuf) -> Result<()> {
    let mut package_json = std::fs::File::create(path.join("package.json"))?;
    writeln!(package_json, r#"
{{
  "name": "tmpCode",
  "version": "1.0.0",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {{
    "build": "tsc"
  }},
  "dependencies": {{
    "@ethersproject/bignumber": "^5.0.6",
    "@ethersproject/bytes": "^5.0.4",
    "int64-buffer": "^0.99.1007",
    "leb": "^0.3.0"
  }},
  "devDependencies": {{
    "@types/node": "12.12.2",
    "typescript": "^3.9.6"
  }}
}}
"#)?;
    let mut tsconfig_json = std::fs::File::create(path.join("tsconfig.json"))?;
    writeln!(tsconfig_json, r#"
{{
  "compilerOptions": {{
    "target": "es6",
    "module": "commonjs",
    "declaration": true,
    "outDir": "./dist",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  }},
  "include": ["testing/*.ts"]
}}
"#)?;
    Ok(())
}

fn test_that_ts_code_compiles_with_config(
    config: &CodeGeneratorConfig,
) -> (TempDir, std::path::PathBuf) {
    let registry = test_utils::get_registry().unwrap();
    let dir = tempdir().unwrap();

    let generator = ts::CodeGenerator::new(&config, true);
    generator
        .write_source_files(dir.path().to_path_buf(), &registry)
        .unwrap();

    let _result = write_package_tsconfig_json_for_test_build(dir.path().to_path_buf());

    let installer = ts::Installer::new(dir.path().to_path_buf());
    installer.install_serde_runtime().unwrap();
    installer.install_bincode_runtime().unwrap();
    installer.install_lcs_runtime().unwrap();

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

#[test]
fn test_that_ts_code_compiles() {
    let config = CodeGeneratorConfig::new("testing".to_string());
    test_that_ts_code_compiles_with_config(&config);
}

#[test]
fn test_that_ts_code_compiles_with_lcs() {
    let config =
        CodeGeneratorConfig::new("testing".to_string()).with_encodings(vec![Encoding::Lcs]);
    test_that_ts_code_compiles_with_config(&config);
}

#[test]
fn test_that_ts_code_compiles_with_comments() {
    let comments = vec![(
        vec!["testing".to_string(), "SerdeData".to_string()],
        "Some\ncomments".to_string(),
    )]
        .into_iter()
        .collect();
    let config = CodeGeneratorConfig::new("testing".to_string()).with_comments(comments);

    let (_dir, path) = test_that_ts_code_compiles_with_config(&config);

    // Comment was correctly generated.
    let content = std::fs::read_to_string(path.join("SerdeData.ts")).unwrap();
    assert!(content.contains(
        r#"
/**
 * Some
 * comments
 */
"#
    ));
}
