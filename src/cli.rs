use clap::Parser;

/// os9s — k9s-inspired terminal UI for kolla-ansible OpenStack
#[derive(Parser, Debug)]
#[command(
    name = "os9s",
    version,
    author,
    about = "k9s-inspired TUI for kolla-ansible provisioned OpenStack",
    long_about = None
)]
pub struct Args {
    /// Path to clouds.yaml file.
    /// Defaults to ~/.config/openstack/clouds.yaml or OS_CLIENT_CONFIG_FILE env var.
    #[arg(
        short = 'f',
        long = "clouds",
        env = "OS_CLIENT_CONFIG_FILE",
        value_name = "PATH"
    )]
    pub clouds: Option<String>,

    /// Cloud name to use from clouds.yaml
    #[arg(
        short = 'c',
        long = "cloud",
        env = "OS_CLOUD",
        default_value = "admin",
        value_name = "CLOUD"
    )]
    pub cloud: Option<String>,

    /// Polling interval in seconds for compute resources (Nova)
    #[arg(long, default_value = "10", value_name = "SECONDS")]
    pub nova_interval: u64,

    /// Polling interval in seconds for network resources (Neutron)
    #[arg(long, default_value = "15", value_name = "SECONDS")]
    pub neutron_interval: u64,

    /// Polling interval in seconds for object storage (Swift)
    #[arg(long, default_value = "30", value_name = "SECONDS")]
    pub swift_interval: u64,

    /// Polling interval in seconds for block storage (Cinder)
    #[arg(long, default_value = "20", value_name = "SECONDS")]
    pub cinder_interval: u64,

    /// Enable verbose debug output to log file
    #[arg(short = 'v', long)]
    pub verbose: bool,
}
