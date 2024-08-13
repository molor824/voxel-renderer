use shaderc::*;
use std::fs::{self, ReadDir};

fn compile_shaders(compiler: &Compiler, options: Option<&CompileOptions>, directory_iter: ReadDir) {
    for entry in directory_iter {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error reading directory:\n{e:?}");
                continue;
            }
        };
        let path = entry.path();
        let shader_kind = match path.extension() {
            Some(extension) => match extension.to_str() {
                Some("vert") => Some(ShaderKind::DefaultVertex),
                Some("frag") => Some(ShaderKind::DefaultFragment),
                Some("comp") => Some(ShaderKind::DefaultCompute),
                Some("glsl") => Some(ShaderKind::InferFromSource),
                _ => None,
            },
            _ => None,
        };
        if let Some(shader_kind) = shader_kind {
            let source = fs::read_to_string(&path).unwrap();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let binary =
                compiler.compile_into_spirv(&source, shader_kind, file_name, "main", options);
            match binary {
                Ok(binary) => {
                    fs::write(format!("target/{file_name}.spv"), binary.as_binary_u8()).unwrap();
                    println!("Compiled {path:?} succesfully.");
                }
                Err(err) => panic!("Error compiling shader {file_name}:\n{err}"),
            }
            continue;
        }
        let Ok(directory) = fs::read_dir(&path) else {
            continue;
        };
        compile_shaders(compiler, options, directory);
    }
}
fn main() {
    println!("cargo::rerun-if-changed=shaders");
    let Ok(directory) = fs::read_dir("shaders") else {
        println!("shaders directory does not exist.");
        return;
    };
    compile_shaders(
        &Compiler::new().expect("Failed to create compiler"),
        Some(&CompileOptions::new().expect("Failed to create compiler options")),
        directory,
    );
}
