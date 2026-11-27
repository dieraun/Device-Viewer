//! Program entry point: GUI by default, batch CLI when asked.

// 每个 `unsafe` 都必须写明前置条件；缺少说明会在 CI 的 `-D warnings` 下直接失败。
#![deny(clippy::undocumented_unsafe_blocks)]

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
    --out <PATH>          Write .txt, .md/.markdown or .json, chosen from the
                          extension; "
    - " writes to standard output.
    --only <LIST>         Comma-separated modules: cpu,memory,pcie,vulkan,opengl,
                          opencl,npu,usb,storage,motherboard,display,directx,metal,
                          openvx,system,network,audio,sensors,bandwidth.
    --snapshot <PATH>     Export an existing .dvsnap/JSON snapshot instead of querying.
    --quiet               Suppress progress and warning chatter.
    --portable            Persist configuration and snapshots beside the executable.
                          Batch mode never writes configuration, so this flag has
                          no effect there.
    --completions <SHELL> Print completion script: bash, zsh or powershell.
    --version             Print the version.
    -h, --help            Print this help.
    --                    Ends the option list; nothing may follow it.

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
        if name == "--" {
            if let Some(argument) = args.next() {
                return Err(format!("unexpected argument `{argument}` after `--`"));
            }
            break;
        }
        if inline.is_some() && is_boolean_flag(&name) {
            return Err(format!("`{name}` does not take a value"));
        }
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

fn is_boolean_flag(name: &str) -> bool {
    matches!(
        name,
        "--cli" | "--quiet" | "-q" | "--portable" | "--version" | "-V" | "--help" | "-h"
    )
}

fn run_cli(cli: &Cli) -> Result<(), String> {
    if !cli.quiet {
        log::init(None);
        log::info("batch mode started");
    }

    let mut snapshot = if let Some(path) = &cli.snapshot {
        let snapshot = read_snapshot(path)?;
        if snapshot.schema_version < state::SCHEMA_VERSION {
            let message = format!(
                "{} was written with schema version {}; reading it as a {} snapshot",
                path.display(),
                snapshot.schema_version,
                state::SCHEMA_VERSION
            );
            log::warn(&message);
            if !cli.quiet {
                eprintln!("note: {message}");
            }
        }
        snapshot
    } else {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| format!("cannot start the async runtime: {error}"))?;
        let cancel = CancellationToken::new();
        runtime.block_on(collect_all(&query_options(cli.only.as_deref()), &cancel))
    };

    if let Some(only) = &cli.only {
        // 只裁剪输出内容；未启用的模块已在采集阶段被门控跳过。
        retain_only(&mut snapshot, only);
        snapshot
            .failures
            .retain(|failure| failure.message != device_query::NOT_ENABLED_MESSAGE);
    }

    let i18n = I18n::new(Language::from_system());
    let units = device_query::UnitSystem::default();

    if let Some(path) = &cli.out {
        let to_stdout = path.as_os_str() == "-";
        let format = if to_stdout {
            ExportFormat::Txt
        } else {
            format_for(path)?
        };
        let body = export::render(&snapshot, format, i18n, units);
        if to_stdout {
            print!("{body}");
        } else {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)
                        .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
                }
            }
            std::fs::write(path, body)
                .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
            if !cli.quiet {
                eprintln!("wrote {}", path.display());
            }
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
    let snapshot = match serde_json::from_slice::<HardwareSnapshot>(&bytes) {
        Ok(snapshot) => snapshot,
        Err(_) => serde_json::from_slice::<Wrapper>(&bytes)
            .map(|wrapper| wrapper.snapshot)
            .map_err(|error| {
                format!(
                    "{} is not a Device Viewer snapshot: {error}",
                    path.display()
                )
            })?,
    };
    // A newer schema would otherwise deserialize partially and look like a
    // machine whose hardware sections are all empty.
    if snapshot.schema_version > state::SCHEMA_VERSION {
        return Err(format!(
            "{} uses schema version {}, but this build understands up to {}; update Device Viewer",
            path.display(),
            snapshot.schema_version,
            state::SCHEMA_VERSION
        ));
    }
    Ok(snapshot)
}

/// The extension decides the format, and an unknown one is an error: silently
/// writing plain text into `report.yaml` produces a file that lies about itself.
fn format_for(path: &PathBuf) -> Result<ExportFormat, String> {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("json") => Ok(ExportFormat::Json),
        Some("md") | Some("markdown") => Ok(ExportFormat::Markdown),
        Some("txt") | Some("text") => Ok(ExportFormat::Txt),
        Some(other) => Err(format!(
            "cannot choose an output format from extension `{other}`; use .txt, .md or .json"
        )),
        None => Err("--out needs a file name with a .txt, .md or .json extension".to_string()),
    }
}

/// `--only` 不只是裁剪输出：未点名的模块必须在采集阶段就被跳过，
/// 否则运行一次 `--only cpu` 仍会探测 GPU、SMART 和带宽。
fn query_options(only: Option<&[String]>) -> QueryOptions {
    let Some(only) = only else {
        return QueryOptions::default();
    };
    let wants = |name: &str| only.iter().any(|module| module == name);
    let mut enabled: Vec<&'static str> = Vec::new();
    for module in device_query::ALL_MODULES {
        // network 与 audio 由 system 模块统一采集，点它们等于点 system。
        let needed = wants(module) || (module == "system" && (wants("network") || wants("audio")));
        if needed {
            enabled.push(module);
        }
    }
    QueryOptions::with_enabled(enabled)
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_limits_the_collected_modules() {
        let options = query_options(Some(&["cpu".to_string()]));
        assert!(options.is_enabled("cpu"));
        assert!(!options.is_enabled("vulkan"));
        assert!(!options.is_enabled("bandwidth"));
    }

    #[test]
    fn network_and_audio_enable_the_system_module() {
        for name in ["network", "audio"] {
            let options = query_options(Some(&[name.to_string()]));
            assert!(
                options.is_enabled("system"),
                "{name} must collect system data"
            );
            assert!(!options.is_enabled("cpu"));
        }
    }

    #[test]
    fn no_selection_runs_every_module() {
        let options = query_options(None);
        for module in device_query::ALL_MODULES {
            assert!(options.is_enabled(module), "{module} must run by default");
        }
    }
}
