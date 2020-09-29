// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use serde_generate::{test_utils, typescript, CodeGeneratorConfig, Encoding, SourceInstaller};
use std::fs::File;
use std::io::{Result, Write};
use std::process::Command;
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
    "forceConsistentCasingInFileNames": true
  }},
  "include": ["testing/*.ts"]
}}
"#
    )?;
    Ok(())
}

pub fn copy_dir<U: AsRef<std::path::Path>, V: AsRef<std::path::Path>>(
    from: U,
    to: V,
) -> Result<()> {
    let mut stack = Vec::new();
    stack.push(std::path::PathBuf::from(from.as_ref()));

    let output_root = std::path::PathBuf::from(to.as_ref());
    let input_root = std::path::PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        println!("process: {:?}", &working_path);

        // Generate a relative path
        let src: std::path::PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if std::fs::metadata(&dest).is_err() {
            println!(" mkdir: {:?}", dest);
            std::fs::create_dir_all(&dest)?;
        }

        for entry in std::fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        println!("  copy: {:?} -> {:?}", &path, &dest_path);
                        std::fs::copy(&path, &dest_path)?;
                    }
                    None => {
                        println!("failed: {:?}", path);
                    }
                }
            }
        }
    }

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
    installer.install_lcs_runtime().unwrap();

    let copy_dir_path = std::path::Path::new("/tmp/shit/");
    std::fs::create_dir_all(&copy_dir_path);
    copy_dir(dir.path(), copy_dir_path);

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
    match std::fs::create_dir_all(dir.path().join("testing")) {
        Ok(_) => {}
        Err(_) => {}
    }
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
