use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "pdown",
    author = "IR-Flutter Community",
    version,
    about = "Native cross-platform Development Environment Mirror Manager",
    long_about = "A reliable, secure, zero-dependency CLI tool to configure and manage package and artifact mirrors for Flutter, Dart, Gradle, and Android."
)]
pub struct Cli {
    #[arg(short, long, global = true, help = "Enable verbose debug logging")]
    pub verbose: bool,

    #[arg(short, long, global = true, help = "Suppress all informational output")]
    pub quiet: bool,

    #[arg(
        long,
        global = true,
        help = "Output results in machine-readable JSON format"
    )]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(
        name = "list",
        alias = "ls",
        about = "List all available mirrors and their capabilities"
    )]
    List(ListArgs),

    #[command(
        name = "status",
        about = "Check current system environment variables and managed configuration files"
    )]
    Status(StatusArgs),

    #[command(
        name = "test",
        alias = "ping",
        about = "Benchmark and test health/latency of mirrors"
    )]
    Test(TestArgs),

    #[command(
        name = "use",
        alias = "activate",
        about = "Configure and activate an environment mirror"
    )]
    Use(UseArgs),

    #[command(
        name = "reset",
        alias = "deactivate",
        about = "Reset mirror settings back to official defaults"
    )]
    Reset(ResetArgs),

    #[command(
        name = "doctor",
        about = "Inspect system health, prerequisites, and detect conflicts"
    )]
    Doctor(DoctorArgs),

    #[command(name = "mirror", about = "Inspect details of a specific mirror")]
    Mirror(MirrorArgs),

    #[command(
        name = "registry",
        about = "Manage the dynamic remote mirror registry and signature verification"
    )]
    Registry(RegistryArgs),

    #[command(
        name = "profile",
        about = "Manage configuration profiles (default, iran, custom)"
    )]
    Profile(ProfileArgs),

    #[command(name = "config", about = "Inspect current active pdown configuration")]
    Config(ConfigArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(
        short,
        long,
        help = "Filter by supported ecosystem (e.g. flutter, dart, gradle, android)"
    )]
    pub provider: Option<String>,
}

#[derive(Args, Debug)]
pub struct StatusArgs {}

#[derive(Args, Debug)]
pub struct TestArgs {
    #[arg(short, long, help = "Specific mirror ID to benchmark")]
    pub mirror: Option<String>,

    #[arg(
        short,
        long,
        default_value = "5",
        help = "HTTP probe timeout in seconds"
    )]
    pub timeout: u64,
}

#[derive(Args, Debug)]
pub struct UseArgs {
    #[arg(
        help = "The unique ID of the mirror to activate (e.g. 'pub-azs', 'runflare', 'flutter-cn')"
    )]
    pub mirror_id: String,

    #[arg(
        long,
        help = "Comma-separated list of providers to apply (e.g. 'flutter,dart' or 'gradle')"
    )]
    pub only: Option<String>,

    #[arg(
        long,
        help = "Preview proposed changes without writing any files or environment variables"
    )]
    pub dry_run: bool,

    #[arg(
        long,
        value_enum,
        default_value_t = ScopeArg::Global,
        help = "Scope of configuration: global (user shell/gradle) or project (current directory)"
    )]
    pub scope: ScopeArg,
}

#[derive(Args, Debug)]
pub struct ResetArgs {
    #[arg(
        long,
        help = "Comma-separated list of providers to reset (e.g. 'flutter' or 'gradle')"
    )]
    pub only: Option<String>,

    #[arg(long, help = "Preview changes without modifying the system")]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct DoctorArgs {}

#[derive(Args, Debug)]
pub struct MirrorArgs {
    #[command(subcommand)]
    pub subcommand: MirrorSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum MirrorSubcommands {
    #[command(
        name = "info",
        about = "Show detailed information about a specific mirror"
    )]
    Info {
        #[arg(help = "Mirror identifier")]
        mirror_id: String,
    },
}

#[derive(Args, Debug)]
pub struct RegistryArgs {
    #[command(subcommand)]
    pub subcommand: RegistrySubcommands,
}

#[derive(Subcommand, Debug)]
pub enum RegistrySubcommands {
    #[command(
        name = "status",
        about = "Show current registry source (bundled, cached, remote), revision, and verification state"
    )]
    Status,

    #[command(
        name = "check",
        about = "Check if an updated registry is available online"
    )]
    Check,

    #[command(
        name = "update",
        about = "Fetch, verify Ed25519 signature, and atomically update cached registry"
    )]
    Update {
        #[arg(long, help = "Force update even if local revision matches remote")]
        force: bool,
    },
}

#[derive(Args, Debug)]
pub struct ProfileArgs {
    #[command(subcommand)]
    pub subcommand: ProfileSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum ProfileSubcommands {
    #[command(name = "list", about = "List all profiles")]
    List,

    #[command(name = "use", about = "Switch active configuration profile")]
    Use { name: String },

    #[command(name = "create", about = "Create a new configuration profile")]
    Create { name: String },
}

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub subcommand: ConfigSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcommands {
    #[command(
        name = "show",
        about = "Display current configuration parameters and storage directories"
    )]
    Show,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum ScopeArg {
    Global,
    Project,
}
