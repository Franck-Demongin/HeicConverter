use crate::cli::{Cli, Command};
use anyhow::{Result, bail};
use clap::Parser;
use heic_converter_core::{ConversionOptions, OutputFormat, convert_file};
use std::fs;
use std::path::{Path, PathBuf};

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Convert {
            inputs,
            output,
            format,
            quality,
            no_overwrite,
        }) => {
            if !output.exists() {
                fs::create_dir_all(&output)?;
            }
            let format = match format.to_lowercase().as_str() {
                "jpeg" | "jpg" => OutputFormat::Jpeg { quality },
                "png" => OutputFormat::Png,
                other => bail!("format inconnu : {other}"),
            };
            if let OutputFormat::Jpeg { quality } = format
                && !(1..=100).contains(&quality)
            {
                bail!("qualite JPEG invalide : {quality}");
            }

            let mut files = Vec::new();
            for input in &inputs {
                if input.is_dir() {
                    collect_heic_files(input, &mut files)?;
                } else if input.is_file() {
                    files.push(input.clone());
                } else {
                    bail!("chemin introuvable : {}", input.display());
                }
            }
            if files.is_empty() {
                bail!("aucun fichier HEIC/HEIF trouve");
            }

            let options = ConversionOptions {
                format,
                apply_transforms: true,
                max_pixels: 1 << 28,
            };

            let mut errors = 0usize;
            for file in &files {
                let out = output_path_for(file, &output, &format);
                if out.exists() && no_overwrite {
                    println!(" ignore {} (deja present)", out.display());
                    continue;
                }
                match convert_file(file, &out, &options) {
                    Ok(result) => println!(
                        " OK {} -> {} ({}x{})",
                        file.display(),
                        out.display(),
                        result.width,
                        result.height
                    ),
                    Err(error) => {
                        errors += 1;
                        eprintln!(" ERR {} : {error}", file.display());
                    }
                }
            }

            if errors > 0 {
                std::process::exit(1);
            }
            Ok(())
        }
        None => {
            println!("HEIC Converter : essayez `heic-converter --help`");
            Ok(())
        }
    }
}

fn output_path_for(input: &Path, output_dir: &Path, format: &OutputFormat) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("sortie");
    let ext = match format {
        OutputFormat::Jpeg { .. } => "jpg",
        OutputFormat::Png => "png",
    };
    output_dir.join(format!("{stem}.{ext}"))
}

fn collect_heic_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_heic_files(&path, files)?;
        } else if is_heic(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn is_heic(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext = ext.to_lowercase();
            ext == "heic" || ext == "heif"
        })
        .unwrap_or(false)
}
