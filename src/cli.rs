use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Launch interactive TUI configuration
    Config,
    /// Generate default config file
    Init,
    /// Check environment compatibility
    Doctor,
    /// Manage themes
    Theme {
        #[command(subcommand)]
        action: ThemeAction,
    },
    /// Apply a preset layout
    Preset {
        /// Preset name: minimal, full, powerline, compact
        name: String,
    },
    /// Dump the expected JSON input schema
    DumpSchema,
}

#[derive(Subcommand)]
pub enum ThemeAction {
    /// List available themes
    List,
    /// Set active theme
    Set { name: String },
}

pub fn handle_command(cmd: Commands) {
    match cmd {
        Commands::Config => {
            eprintln!("TUI config not yet implemented");
        }
        Commands::Init => {
            eprintln!("Init not yet implemented");
        }
        Commands::Doctor => {
            eprintln!("Doctor not yet implemented");
        }
        Commands::Theme { action } => match action {
            ThemeAction::List => {
                eprintln!("Theme list not yet implemented");
            }
            ThemeAction::Set { name } => {
                eprintln!("Theme set to: {name}");
            }
        },
        Commands::Preset { name } => {
            eprintln!("Preset '{name}' not yet implemented");
        }
        Commands::DumpSchema => {
            eprintln!("Schema dump not yet implemented");
        }
    }
}
