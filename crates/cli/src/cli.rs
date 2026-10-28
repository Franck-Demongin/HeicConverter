use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "heic-converter",
    version,
    about = "Conversion d'images HEIC/HEIF vers JPEG ou PNG"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Convertit des fichiers HEIC/HEIF.
    Convert {
        /// Fichiers ou dossiers a convertir.
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        /// Repertoire de sortie.
        #[arg(short, long)]
        output: PathBuf,
        /// Format de sortie (jpeg ou png).
        #[arg(long, default_value = "jpeg")]
        format: String,
        /// Qualite JPEG (1..=100).
        #[arg(long, default_value_t = 90)]
        quality: u8,
        /// Ne pas ecraser les fichiers existants.
        #[arg(long)]
        no_overwrite: bool,
    },
}
