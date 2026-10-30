//! Program entry point: GUI by default, batch CLI when asked.

mod app;
mod device_query;
mod export;
mod i18n;
mod log;
mod state;
mod ui;

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::ExitCode;

use device_query::{HardwareSnapshot, QueryOptions, collect_all};
use export::ExportFormat;
use i18n::{I18n, Language};
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

const HELP: &str = "\
Device Viewer — cross-platform hardware inspector

USAGE:
    device-viewer [OPTIONS]

OPTIONS:
    --cli                 Force the batch/CLI mode (implied by --out/--only/--snapshot).
    --out <PATH>          Write TXT, Markdown or JSON, chosen from the extension.
    --only <LIST>         Comma-separated modules: cpu,memory,pcie,vulkan,opengl,
                          opencl,npu,usb,storage,motherboard,display,directx,metal,
                          openvx,system,network,audio,sensors,bandwidth.
    --snapshot <PATH>     Export an existing .dvsnap/JSON snapshot instead of querying.
    --quiet               Suppress progress and warning chatter.
    --portable            Persist configuration and snapshots beside the executable.
    --completions <SHELL> Print completion script: bash, zsh or powershell.
    --version             Print the version.
    -h, --help            Print this help.

The GUI starts when no batch option is supplied.";

#[derive(Debug, Default)]
struct Cli {
    cli: bool,
    out: Option<PathBuf>,
    only: Option<Vec<String>>,
    quiet: bool,
    portable: bool,
    snapshot: Option<PathBuf>,
    completions: Option<String>,
    version: bool,
    help: bool,
}

fn main() -> ExitCode {
    let cli = match parse_args(std::env::args().skip(1)) {
        Ok(cli) => cli,
        Err(error) => {
            eprintln!("error: {error}\n\n{HELP}");
            return ExitCode::FAILURE;
        }
    };

    if cli.help {
        println!("{HELP}");
        return ExitCode::SUCCESS;
    }
    if cli.version {
        println!("device-viewer {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    if let Some(shell) = &cli.completions {
        match completion_script(shell) {
            Ok(script) => {
                print!("{script}");
                return ExitCode::SUCCESS;
            }
            Err(error) => {
                eprintln!("error: {error}");
                return ExitCode::FAILURE;
            }
        }
    }

    let batch = cli.cli || cli.out.is_some() || cli.only.is_some() || cli.snapshot.is_some();

    if batch {
        match run_cli(&cli) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::FAILURE
            }
        }
    } else {
        match app::run(cli.portable) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("error: failed to start the GUI: {error}");
                ExitCode::FAILURE
            }
        }
    }
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Cli, String> {
    let mut cli = Cli::default();
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        let (name, inline) = match argument.split_once('=') {
            Some((name, value)) => (name.to_string(), Some(value.to_string())),
            None => (argument, None),
        };
        let mut take = |label: &str| -> Result<String, String> {
            if let Some(value) = inline.clone() {
                return Ok(value);
            }
            args.next()
                .ok_or_else(|| format!("{label} requires a value"))
        };
        match name.as_str() {
            "--cli" => cli.cli = true,
            "--quiet" | "-q" => cli.quiet = true,
            "--portable" => cli.portable = true,
            "--version" | "-V" => cli.version = true,
            "--help" | "-h" => cli.help = true,
            "--out" | "-o" => cli.out = Some(PathBuf::from(take("--out")?)),
            "--snapshot" => cli.snapshot = Some(PathBuf::from(take("--snapshot")?)),
            "--completions" => cli.completions = Some(take("--completions")?),
            "--only" => {
                let value = take("--only")?;
                let modules: Vec<String> = value
                    .split(',')
                    .map(|module| module.trim().to_ascii_lowercase())
                    .filter(|module| !module.is_empty())
                    .collect();
                if modules.is_empty() {
                    return Err("--only needs at least one module".to_string());
                }
                const MODULES: [&str; 19] = [
                    "cpu",
                    "memory",
                    "pcie",
                    "vulkan",
                    "opengl",
                    "opencl",
                    "npu",
                    "usb",
                    "storage",
                    "motherboard",
                    "display",
                    "directx",
                    "metal",
                    "openvx",
                    "system",
                    "network",
                    "audio",
                    "sensors",
                    "bandwidth",
                ];
                if let Some(unknown) = modules
                    .iter()
                    .find(|module| !MODULES.contains(&module.as_str()))
                {
                    return Err(format!("unknown module for --only: `{unknown}`"));
                }
                cli.only = Some(modules);
            }
            other => return Err(format!("unknown option `{other}`")),
        }
    }
    Ok(cli)
}

fn run_cli(cli: &Cli) -> Result<(), String> {
    if !cli.quiet {
        log::init(None);
        log::info("batch mode started");
    }

    let mut snapshot = if let Some(path) = &cli.snapshot {
        read_snapshot(path)?
    } else {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|error| format!("cannot start the async runtime: {error}"))?;
        let cancel = CancellationToken::new();
        runtime.block_on(collect_all(&QueryOptions::default(), &cancel))
    };

    if let Some(only) = &cli.only {
        retain_only(&mut snapshot, only);
    }

    let i18n = I18n::new(Language::from_system());
    let units = device_query::UnitSystem::default();

    if let Some(path) = &cli.out {
        let format = format_for(path);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
            }
        }
        let body = export::render(&snapshot, format, i18n, units);
        std::fs::write(path, body)
            .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
        if !cli.quiet {
            eprintln!("wrote {}", path.display());
        }
    } else {
        print!(
            "{}",
            export::render(&snapshot, ExportFormat::Txt, i18n, units)
        );
    }

    if !cli.quiet {
        for failure in &snapshot.failures {
            eprintln!(
                "{}: {} ({})",
                failure.module,
                failure.message,
                failure.level.as_str()
            );
        }
    }
    Ok(())
}

fn read_snapshot(path: &PathBuf) -> Result<HardwareSnapshot, String> {
    let bytes =
        std::fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    #[derive(Deserialize)]
    struct Wrapper {
        snapshot: HardwareSnapshot,
    }
    if let Ok(snapshot) = serde_json::from_slice::<HardwareSnapshot>(&bytes) {
        return Ok(snapshot);
    }
    serde_json::from_slice::<Wrapper>(&bytes)
        .map(|wrapper| wrapper.snapshot)
        .map_err(|error| {
            format!(
                "{} is not a Device Viewer snapshot: {error}",
                path.display()
            )
        })
}

fn format_for(path: &PathBuf) -> ExportFormat {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("json") => ExportFormat::Json,
        Some("md") | Some("markdown") => ExportFormat::Markdown,
        _ => ExportFormat::Txt,
    }
}

fn retain_only(snapshot: &mut HardwareSnapshot, only: &[String]) {
    let selected: BTreeSet<&str> = only.iter().map(String::as_str).collect();
    let keep = |name: &str| selected.contains(name);
    if !keep("cpu") {
        snapshot.cpu = None;
    }
    if !keep("memory") {
        snapshot.memory = None;
    }
    if !keep("pcie") {
        snapshot.pcie = None;
    }
    if !keep("vulkan") {
        snapshot.vulkan = None;
    }
    if !keep("opengl") {
        snapshot.opengl = None;
    }
    if !keep("opencl") {
        snapshot.opencl = None;
    }
    if !keep("npu") {
        snapshot.npu = None;
    }
    if !keep("usb") {
        snapshot.usb = None;
    }
    if !keep("storage") {
        snapshot.storage = None;
    }
    if !keep("motherboard") {
        snapshot.motherboard = None;
    }
    if !keep("display") {
        snapshot.display = None;
    }
    if !keep("directx") {
        snapshot.directx = None;
    }
    if !keep("metal") {
        snapshot.metal = None;
    }
    if !keep("openvx") {
        snapshot.openvx = None;
    }
    if !keep("system") && !keep("network") && !keep("audio") {
        snapshot.system = None;
    }
    if !keep("sensors") {
        snapshot.sensors = None;
    }
    if !keep("bandwidth") {
        snapshot.bandwidth = None;
    }
    snapshot.failures.retain(|failure| {
        keep(failure.module.as_str())
            || (failure.module == "system" && (keep("network") || keep("audio")))
    });
}

fn completion_script(shell: &str) -> Result<String, String> {
    let shell = shell.to_ascii_lowercase();
    let script = match shell.as_str() {
        "bash" => r#"# bash completion for device-viewer
_device_viewer() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local opts="--cli --out --only --snapshot --quiet --portable --completions --version --help"
    local modules="cpu,memory,pcie,vulkan,opengl,opencl,npu,usb,storage,motherboard,display,directx,metal,openvx,system,network,audio,sensors,bandwidth"
    case "${COMP_WORDS[COMP_CWORD-1]}" in
        --out|-o|--snapshot) COMPREPLY=( $(compgen -f -- "$cur") ); return ;;
        --completions) COMPREPLY=( $(compgen -W "bash zsh powershell" -- "$cur") ); return ;;
        --only) COMPREPLY=( $(compgen -W "$modules" -- "$cur") ); return ;;
    esac
    COMPREPLY=( $(compgen -W "$opts" -- "$cur") )
}
complete -F _device_viewer device-viewer
"#
        .to_string(),
        "zsh" => r#"#compdef device-viewer
_device_viewer() {
    _arguments \
        '--cli[Batch mode]' \
        '--out[Output file]:file:_files' \
        '--only[Modules to include]:modules:(cpu memory pcie vulkan opengl opencl npu usb storage motherboard display directx metal openvx system network audio sensors bandwidth)' \
        '--snapshot[Snapshot to export]:file:_files' \
        '--quiet[Suppress chatter]' \
        '--portable[Portable mode]' \
        '--completions[Print completion script]:shell:(bash zsh powershell)' \
        '--version[Print version]' \
        '--help[Print help]'
}
compdef _device_viewer device-viewer
"#
        .to_string(),
        "powershell" | "pwsh" => r#"# PowerShell completion for device-viewer
Register-ArgumentCompleter -Native -CommandName device-viewer -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    $options = @('--cli','--out','--only','--snapshot','--quiet','--portable','--completions','--version','--help')
    $modules = @('cpu','memory','pcie','vulkan','opengl','opencl','npu','usb','storage','motherboard','display','directx','metal','openvx','system','network','audio','sensors','bandwidth')
    $options + $modules | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
    }
}
"#
        .to_string(),
        other => return Err(format!("unsupported shell `{other}`; use bash, zsh or powershell")),
    };
    Ok(script)
}
