use crate::app::cli::{
    Cli, Commands, ListArgs, MirrorArgs, MirrorSubcommands, RegistryArgs, RegistrySubcommands,
};
use crate::core::errors::{PdownError, Result};
use crate::core::registry::RegistryManager;
use crate::infrastructure::storage::StorageManager;

pub async fn execute(cli: Cli) -> Result<()> {
    let storage = StorageManager::new()?;
    let registry = RegistryManager::new(storage);

    match cli.command {
        Some(Commands::List(args)) => handle_list(&registry, args, cli.json).await,
        Some(Commands::Mirror(args)) => handle_mirror(&registry, args, cli.json).await,
        Some(Commands::Registry(args)) => handle_registry(&registry, args, cli.json).await,
        Some(Commands::Status(args)) => handle_status(args, cli.json).await,
        Some(Commands::Test(_args)) => {
            println!("pdown test - available in Phase 5");
            Ok(())
        }
        Some(Commands::Use(args)) => {
            println!("pdown use {} - available in Phase 4", args.mirror_id);
            Ok(())
        }
        Some(Commands::Reset(_args)) => {
            println!("pdown reset - available in Phase 4");
            Ok(())
        }
        Some(Commands::Doctor(_args)) => {
            println!("pdown doctor - available in Phase 6");
            Ok(())
        }
        Some(Commands::Profile(_args)) => {
            println!("pdown profile - available in Phase 6");
            Ok(())
        }
        Some(Commands::Config(_args)) => {
            println!("pdown config - available in Phase 6");
            Ok(())
        }
        None => {
            use clap::CommandFactory;
            Cli::command().print_help().unwrap();
            println!();
            Ok(())
        }
    }
}

async fn handle_status(_args: crate::app::cli::StatusArgs, json: bool) -> Result<()> {
    let providers = crate::providers::all_providers();
    let mut statuses = Vec::new();

    for p in providers {
        statuses.push(p.detect_current()?);
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&statuses)?);
        return Ok(());
    }

    println!("\n📊 Current System & Ecosystem Mirror Status:\n");
    for s in &statuses {
        let active_badge = if s.is_active {
            "🟢 Active (Mirrored)"
        } else {
            "⚪ Inactive (Upstream Default)"
        };
        println!("🔹 {} [{}]", s.provider_id.to_uppercase(), active_badge);
        for (k, v) in &s.details {
            println!("   {}: {}", k, v);
        }
        println!();
    }
    Ok(())
}

async fn handle_list(registry: &RegistryManager, args: ListArgs, json: bool) -> Result<()> {
    let mirrors = registry.list_mirrors(args.provider.as_deref())?;

    if json {
        println!("{}", serde_json::to_string_pretty(&mirrors)?);
        return Ok(());
    }

    println!("\n📋 Available Development Environment Mirrors:\n");
    for m in &mirrors {
        let mut caps = Vec::new();
        if m.providers.flutter.as_ref().is_some_and(|p| p.enabled) {
            caps.push("Flutter");
        }
        if m.providers.dart.as_ref().is_some_and(|p| p.enabled) {
            caps.push("Dart/Pub");
        }
        if m.providers.gradle.as_ref().is_some_and(|p| p.enabled) {
            caps.push("Gradle");
        }
        if m.providers.android.as_ref().is_some_and(|p| p.enabled) {
            caps.push("Android");
        }

        let coverage = if caps.is_empty() {
            "None".to_string()
        } else {
            caps.join(" + ")
        };

        println!("🔹 [{}] {} ({})", m.id, m.name, coverage);
        println!("   Description: {}", m.description);
        println!("   Website:     {}", m.website);

        if let Some(dart) = &m.providers.dart {
            if dart.enabled {
                println!("   Pub Hosted:  {}", dart.hosted_url);
            }
        }
        if let Some(flutter) = &m.providers.flutter {
            if flutter.enabled {
                println!("   Flutter GCS: {}", flutter.storage_url);
            }
        }
        if let Some(gradle) = &m.providers.gradle {
            if gradle.enabled {
                if let Some(m_central) = &gradle.maven_central_url {
                    println!("   Maven:       {}", m_central);
                }
                if let Some(dist) = &gradle.distribution_url {
                    println!("   Gradle Dist: {}", dist);
                }
            }
        }
        println!("   ----------------------------------------");
    }
    println!("\nTo activate a mirror, run: pdown use <mirror-id>");
    Ok(())
}

async fn handle_mirror(registry: &RegistryManager, args: MirrorArgs, json: bool) -> Result<()> {
    match args.subcommand {
        MirrorSubcommands::Info { mirror_id } => {
            let mirror = registry
                .find_mirror(&mirror_id)?
                .ok_or_else(|| PdownError::MirrorNotFound(mirror_id.clone()))?;

            if json {
                println!("{}", serde_json::to_string_pretty(&mirror)?);
            } else {
                println!("\n🔍 Mirror Details: [{}]", mirror.id);
                println!("   Name:        {}", mirror.name);
                println!("   Description: {}", mirror.description);
                println!("   Website:     {}", mirror.website);
                println!("   Enabled:     {}", mirror.enabled);
                println!(
                    "   Priority:    {}",
                    mirror
                        .priority
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "Default (100)".into())
                );
                println!("   Providers:");
                if let Some(f) = mirror.providers.flutter {
                    println!(
                        "     • Flutter Storage: enabled={}, url={}",
                        f.enabled, f.storage_url
                    );
                }
                if let Some(d) = mirror.providers.dart {
                    println!(
                        "     • Dart/Pub:        enabled={}, url={}",
                        d.enabled, d.hosted_url
                    );
                }
                if let Some(g) = mirror.providers.gradle {
                    println!(
                        "     • Gradle:          enabled={}, maven_central={:?}, google_maven={:?}, dist={:?}",
                        g.enabled, g.maven_central_url, g.google_maven_url, g.distribution_url
                    );
                }
                if let Some(a) = mirror.providers.android {
                    println!(
                        "     • Android:         enabled={}, google_maven={:?}",
                        a.enabled, a.google_maven_url
                    );
                }
                if let Some(i) = mirror.providers.ios {
                    println!(
                        "     • iOS:             enabled={}, notes={:?}",
                        i.enabled, i.notes
                    );
                }
            }
            Ok(())
        }
    }
}

async fn handle_registry(registry: &RegistryManager, args: RegistryArgs, json: bool) -> Result<()> {
    match args.subcommand {
        RegistrySubcommands::Status => {
            let status = registry.status()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else {
                println!("\n📦 Remote Registry Status:");
                println!("   Active Source:      {}", status.active_source);
                println!("   Registry Version:   v{}", status.registry_version);
                println!("   Revision:           {}", status.revision);
                println!("   Schema Version:     v{}", status.schema_version);
                println!("   Total Mirrors:      {}", status.mirrors_count);
                println!(
                    "   Signature Valid:    {}",
                    if status.is_signature_valid {
                        "✅ Yes (Ed25519)"
                    } else {
                        "❌ No"
                    }
                );
                println!("   Last Updated:       {}", status.updated_at);
            }
            Ok(())
        }
        RegistrySubcommands::Check => {
            let check = registry.check_remote().await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&check)?);
            } else if check.update_available {
                println!(
                    "✨ New registry update available! Current: rev {} (v{}), Remote: rev {} (v{})",
                    check.current_revision,
                    check.current_version,
                    check.remote_revision,
                    check.remote_version
                );
                println!("Run 'pdown registry update' to download and verify.");
            } else {
                println!(
                    "✅ Registry is up to date (current rev {}, version v{}).",
                    check.current_revision, check.current_version
                );
            }
            Ok(())
        }
        RegistrySubcommands::Update { force } => {
            let update = registry.update_remote(force).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&update)?);
            } else {
                println!(
                    "🎉 Registry successfully updated from revision {} to {} (v{})",
                    update.previous_revision, update.new_revision, update.new_version
                );
            }
            Ok(())
        }
    }
}
