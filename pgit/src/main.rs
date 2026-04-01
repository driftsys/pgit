use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "pgit")]
#[command(about = "Git-based package registry")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Install a package from a registry
    Add {
        /// PURL or shorthand: github.com/owner/repo/name@version
        source: String,
        /// Target directory (default: current directory)
        #[arg(short, long)]
        out: Option<String>,
        /// Git ref, branch, or tag (overrides version in source)
        #[arg(long, short = 'r')]
        git_ref: Option<String>,
        /// Auth token (or set PGIT_TOKEN / GITHUB_TOKEN / GITLAB_TOKEN)
        #[arg(long)]
        token: Option<String>,
    },

    /// Publish a package to a registry
    Publish {
        /// Path to the package directory
        path: String,
        /// Registry PURL or shorthand: github.com/owner/repo
        #[arg(long)]
        to: String,
        /// Version to publish (semver)
        #[arg(long, short = 'v')]
        version: String,
        /// Create a GitHub/GitLab Release with the package archive attached
        #[arg(long)]
        release: bool,
        /// Auth token
        #[arg(long)]
        token: Option<String>,
    },

    /// Verify installed packages against their recorded content hashes
    Verify {
        /// Package names to verify (omit for all)
        names: Vec<String>,
        /// Exit non-zero on any mismatch (default: always)
        #[arg(long)]
        strict: bool,
    },

    /// List packages in a registry
    List {
        /// Registry PURL or shorthand
        registry: String,
        /// Auth token
        #[arg(long)]
        token: Option<String>,
    },

    /// Create an immutable archive snapshot of a package
    Archive {
        /// PURL or shorthand
        source: String,
        /// Output directory
        #[arg(long, short = 'o')]
        out: String,
        /// Auth token
        #[arg(long)]
        token: Option<String>,
    },

    /// Initialise a new registry repository
    Init {
        /// Registry name
        #[arg(long)]
        name: String,
        /// Archive mode (append-only, no overwrites)
        #[arg(long)]
        archive: bool,
        /// Initialise git-lfs tracking for binary patterns
        #[arg(long)]
        lfs: bool,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { source, out, git_ref, token } => {
            cmd_add(&source, out.as_deref(), git_ref.as_deref(), token.as_deref())
        }
        Commands::Publish { path, to, version, release, token } => {
            cmd_publish(&path, &to, &version, release, token.as_deref())
        }
        Commands::Verify { names, strict } => cmd_verify(&names, strict),
        Commands::List { registry, token } => cmd_list(&registry, token.as_deref()),
        Commands::Archive { source, out, token } => {
            cmd_archive(&source, &out, token.as_deref())
        }
        Commands::Init { name, archive, lfs } => cmd_init(&name, archive, lfs),
    }
}

fn cmd_add(
    _source:  &str,
    _out:     Option<&str>,
    _git_ref: Option<&str>,
    _token:   Option<&str>,
) -> Result<()> {
    todo!("pgit add")
}

fn cmd_publish(
    _path:    &str,
    _to:      &str,
    _version: &str,
    _release: bool,
    _token:   Option<&str>,
) -> Result<()> {
    todo!("pgit publish")
}

fn cmd_verify(_names: &[String], _strict: bool) -> Result<()> {
    todo!("pgit verify")
}

fn cmd_list(_registry: &str, _token: Option<&str>) -> Result<()> {
    todo!("pgit list")
}

fn cmd_archive(_source: &str, _out: &str, _token: Option<&str>) -> Result<()> {
    todo!("pgit archive")
}

fn cmd_init(_name: &str, _archive: bool, _lfs: bool) -> Result<()> {
    todo!("pgit init")
}
