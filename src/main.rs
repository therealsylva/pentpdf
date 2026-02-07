use anyhow::{Context, Result, bail};
use clap::Parser;
use lopdf::Document;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "pentpdf")]
#[command(author = "pentpdf", version = "0.1.0", about = "Splits PDFs into manageable chunks.", long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    #[arg(short, long, default_value = ".", value_name = "DIR")]
    output_dir: PathBuf,

    #[arg(short, long, default_value_t = 100, value_name = "NUM")]
    pages: usize,

    #[arg(short, long, default_value = "output", value_name = "PREFIX")]
    prefix: String,
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if !cli.input.exists() {
        bail!("Input file not found: {:?}", cli.input);
    }
    if !cli.input.is_file() {
        bail!("Input path is not a file: {:?}", cli.input);
    }

    if !cli.output_dir.exists() {
        fs::create_dir_all(&cli.output_dir)
            .context("Failed to create output directory")?;
    }

    let total_pages = {
        let doc = Document::load(&cli.input).context("Failed to parse PDF file. Is it a valid PDF?")?;
        doc.get_pages().len()
    };

    println!("Loaded PDF: {:?} (Total pages: {})", cli.input, total_pages);

    if total_pages <= cli.pages {
        println!(
            "File has {} pages, which is not greater than the limit of {}. No split needed.",
            total_pages, cli.pages
        );
        return Ok(());
    }

    let chunks: Vec<(usize, usize)> = (0..total_pages)
        .step_by(cli.pages)
        .map(|start| (start, (start + cli.pages).min(total_pages)))
        .collect();

    println!(
        "Splitting into {} part(s) (max {} pages per file)...",
        chunks.len(),
        cli.pages
    );

    for (i, (start_page, end_page)) in chunks.iter().enumerate() {
        let part_index = i + 1;
        
        let filename = format!("{}_part{}.pdf", cli.prefix, part_index);
        let output_path = cli.output_dir.join(&filename);

        print!(
            "Writing [{}] (pages {}-{})... ",
            filename,
            start_page + 1,
            end_page
        );

        let mut doc = Document::load(&cli.input)
            .with_context(|| format!("Failed to reload PDF for part {}", part_index))?;

        let pages_to_remove: Vec<u32> = (0..*start_page as u32)
            .chain((*end_page as u32)..(total_pages as u32))
            .collect();

        if !pages_to_remove.is_empty() {
            doc.delete_pages(&pages_to_remove);
        }

        doc.save(&output_path)
            .with_context(|| format!("Failed to write output file: {:?}", output_path))?;

        println!("Done.");
    }

    println!("All parts saved successfully to: {:?}", cli.output_dir);
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("[Error] {}", e);
        std::process::exit(1);
    }
}
