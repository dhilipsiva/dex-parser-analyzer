//! `dexpa`: the summary and static-analysis findings of an Android DEX file or APK.

use std::{cmp::Reverse, fs, path::PathBuf, process::ExitCode};

use anyhow::{Context, Result, bail};
use clap::{Parser, ValueEnum};
use dexpa_analysis::{
    config::AnalysisConfig,
    engine::analyze_dex,
    model::{AnalysisReport, Severity},
};
use dexpa_core::{dto::dex_to_overview, multidex::read_dex_buffers_from_bytes, parse_dex};
use serde::Serialize;

/// The version of the `--format json` output; it changes only with a breaking change to its shape.
const SCHEMA_VERSION: u32 = 1;

/// How many classes the summary lists.
const TOP_CLASSES: usize = 10;

#[derive(Debug, Parser)]
#[command(
    name = "dexpa",
    author,
    version,
    about = "Summarise and statically analyse an Android DEX file or APK"
)]
struct Args {
    /// A `.dex` file, or an `.apk`/zip whose `classes*.dex` entries are analysed in name order.
    #[arg(value_name = "FILE")]
    input: PathBuf,

    /// Output format.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,

    /// At most this many findings per DEX file.
    #[arg(long, value_name = "COUNT")]
    max_findings: Option<usize>,

    /// Exit with status 2 when any finding is at or above this severity.
    #[arg(long, value_enum, value_name = "SEVERITY")]
    fail_on: Option<Threshold>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Format {
    Text,
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Threshold {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Threshold {
    fn admits(self, severity: Severity) -> bool {
        rank(severity) >= self as u8
    }
}

fn rank(severity: Severity) -> u8 {
    match severity {
        Severity::Info => 0,
        Severity::Low => 1,
        Severity::Medium => 2,
        Severity::High => 3,
        Severity::Critical => 4,
    }
}

/// The `--format json` document (`schema_version` 1).
#[derive(Serialize)]
struct Output {
    schema_version: u32,
    input: String,
    dex_files: Vec<DexOutput>,
}

#[derive(Serialize)]
struct DexOutput {
    /// Position in the input: 0 for a single DEX, else the order of the APK's `classes*.dex` entries.
    index: usize,
    summary: Summary,
    top_classes: Vec<TopClass>,
    report: AnalysisReport,
}

#[derive(Serialize)]
struct Summary {
    version: String,
    checksum: u32,
    file_size: u32,
    string_count: u32,
    type_count: u32,
    field_count: u32,
    method_count: u32,
    class_count: u32,
}

#[derive(Serialize)]
struct TopClass {
    descriptor: String,
    method_count: u32,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args) {
        Ok(output) => {
            let tripped = args.fail_on.is_some_and(|threshold| {
                output
                    .dex_files
                    .iter()
                    .flat_map(|dex| &dex.report.findings)
                    .any(|finding| threshold.admits(finding.severity))
            });
            let printed = match args.format {
                Format::Text => {
                    print_text(&output);
                    Ok(())
                }
                Format::Json => serde_json::to_string_pretty(&output)
                    .map(|json| println!("{json}"))
                    .context("failed to serialise the report"),
            };
            match printed {
                Err(err) => fail(&err),
                Ok(()) if tripped => ExitCode::from(2),
                Ok(()) => ExitCode::SUCCESS,
            }
        }
        Err(err) => fail(&err),
    }
}

fn fail(err: &anyhow::Error) -> ExitCode {
    eprintln!("Error: {err:?}");
    ExitCode::from(1)
}

fn run(args: &Args) -> Result<Output> {
    let bytes = fs::read(&args.input)
        .with_context(|| format!("failed to read {}", args.input.display()))?;
    let buffers = if bytes.starts_with(b"PK\x03\x04") {
        let buffers = read_dex_buffers_from_bytes(&bytes).context("failed to read the APK")?;
        if buffers.is_empty() {
            bail!("{} contains no classes*.dex entry", args.input.display());
        }
        buffers
    } else {
        vec![bytes]
    };

    let mut config = AnalysisConfig::default();
    if args.max_findings.is_some() {
        config.max_findings = args.max_findings;
    }
    let dex_files = buffers
        .iter()
        .enumerate()
        .map(|(index, buffer)| analyse(index, buffer, &config))
        .collect::<Result<Vec<_>>>()?;
    Ok(Output {
        schema_version: SCHEMA_VERSION,
        input: args.input.display().to_string(),
        dex_files,
    })
}

fn analyse(index: usize, bytes: &[u8], config: &AnalysisConfig) -> Result<DexOutput> {
    let dex = parse_dex(bytes).with_context(|| format!("failed to parse DEX file {index}"))?;
    let overview = dex_to_overview(&dex).context("failed to build the DEX summary")?;
    let mut classes = overview
        .classes
        .iter()
        .map(|class| TopClass {
            descriptor: class.descriptor.clone(),
            method_count: class.method_count,
        })
        .collect::<Vec<_>>();
    classes.sort_by_key(|class| Reverse(class.method_count));
    classes.truncate(TOP_CLASSES);
    Ok(DexOutput {
        index,
        summary: Summary {
            version: overview.version,
            checksum: overview.checksum,
            file_size: overview.file_size,
            string_count: overview.string_count,
            type_count: overview.type_count,
            field_count: overview.field_count,
            method_count: overview.method_count,
            class_count: overview.class_count,
        },
        top_classes: classes,
        report: analyze_dex(&dex, config),
    })
}

fn print_text(output: &Output) {
    for dex in &output.dex_files {
        if output.dex_files.len() > 1 {
            println!(
                "== DEX file {} of {}\n",
                dex.index + 1,
                output.dex_files.len()
            );
        }
        let summary = &dex.summary;
        println!("Dex summary for {}\n", output.input);
        println!("Version:        {}", summary.version);
        println!("Checksum:       {}", summary.checksum);
        println!("File size:      {} bytes", summary.file_size);
        println!("Strings:        {}", summary.string_count);
        println!("Types:          {}", summary.type_count);
        println!("Fields:         {}", summary.field_count);
        println!("Methods:        {}", summary.method_count);
        println!("Classes:        {}", summary.class_count);

        println!("\nTop classes by method count:");
        for class in &dex.top_classes {
            println!("  {:<5} {}", class.method_count, class.descriptor);
        }
        if dex.top_classes.is_empty() {
            println!("  (no classes decoded)");
        }

        let report = &dex.report;
        println!(
            "\nAnalysis stats: methods={} strings={} instructions={} findings={}",
            report.stats.method_count,
            report.stats.string_count,
            report.stats.instruction_count,
            report.findings.len()
        );
        if let Some(ms) = report.stats.elapsed_ms {
            println!("Analysis elapsed: {ms} ms");
        }

        println!("\nAnalysis findings ({} total):", report.findings.len());
        for (idx, finding) in report.findings.iter().enumerate() {
            let loc = &finding.location;
            println!(
                "{:>3}. [{:?}] {} :: {}::{}",
                idx + 1,
                finding.severity,
                finding.id,
                loc.class_descriptor,
                loc.method_name,
            );
            println!("     {}", finding.message);
            if let Some(pc) = loc.pc {
                println!("     @pc {pc}");
            }
        }
        if report.findings.is_empty() {
            println!("  No findings reported.");
        }
        println!();
    }
}
