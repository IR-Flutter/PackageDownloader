use crate::app::cli::{
    Cli, Commands, ListArgs, MirrorArgs, MirrorSubcommands, RegistryArgs, RegistrySubcommands,
};
use crate::core::errors::{PdownError, Result};
use crate::core::provider::Provider;
use crate::core::registry::RegistryManager;
use crate::infrastructure::storage::StorageManager;

pub async fn execute(cli: Cli) -> Result<()> {
    let storage = StorageManager::new()?;
    let registry = RegistryManager::new(storage.clone());

    match cli.command {
        Some(Commands::List(args)) => handle_list(&registry, args, cli.json).await,
        Some(Commands::Mirror(args)) => handle_mirror(&registry, args, cli.json).await,
        Some(Commands::Registry(args)) => handle_registry(&registry, args, cli.json).await,
        Some(Commands::Status(args)) => handle_status(args, cli.json).await,
        Some(Commands::Test(args)) => handle_test(&registry, args, cli.json).await,
        Some(Commands::Use(args)) => handle_use(&registry, &storage, args, cli.json).await,
        Some(Commands::Reset(args)) => handle_reset(&storage, args, cli.json).await,
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

async fn handle_use(
    registry: &RegistryManager,
    storage: &StorageManager,
    args: crate::app::cli::UseArgs,
    json: bool,
) -> Result<()> {
    let mirror = registry
        .find_mirror(&args.mirror_id)?
        .ok_or_else(|| PdownError::MirrorNotFound(args.mirror_id.clone()))?;

    let only_set: Option<Vec<String>> = args
        .only
        .map(|s| s.split(',').map(|p| p.trim().to_lowercase()).collect());

    let has_flutter = only_set
        .as_ref()
        .map_or(true, |f| f.contains(&"flutter".to_string()))
        && mirror.providers.flutter.as_ref().is_some_and(|f| f.enabled);

    let has_dart = only_set.as_ref().map_or(true, |d| {
        d.contains(&"dart".to_string()) || d.contains(&"pub".to_string())
    }) && mirror.providers.dart.as_ref().is_some_and(|d| d.enabled);

    let has_gradle = only_set
        .as_ref()
        .map_or(true, |g| g.contains(&"gradle".to_string()))
        && mirror.providers.gradle.as_ref().is_some_and(|g| g.enabled);

    let has_android = only_set
        .as_ref()
        .map_or(true, |a| a.contains(&"android".to_string()))
        && mirror.providers.android.as_ref().is_some_and(|a| a.enabled);

    // If user explicitly asked for a provider via --only, verify it's supported
    if let Some(ref filter) = only_set {
        for target in filter {
            match target.as_str() {
                "flutter" if !has_flutter => {
                    return Err(PdownError::ProviderUnsupported {
                        provider: "flutter".into(),
                        reason: format!(
                            "Mirror '{}' does not support Flutter storage artifacts",
                            mirror.id
                        ),
                    });
                }
                "dart" | "pub" if !has_dart => {
                    return Err(PdownError::ProviderUnsupported {
                        provider: "dart".into(),
                        reason: format!(
                            "Mirror '{}' does not support Dart/Pub packages",
                            mirror.id
                        ),
                    });
                }
                "gradle" if !has_gradle => {
                    return Err(PdownError::ProviderUnsupported {
                        provider: "gradle".into(),
                        reason: format!(
                            "Mirror '{}' does not support Gradle repositories",
                            mirror.id
                        ),
                    });
                }
                "android" if !has_android => {
                    return Err(PdownError::ProviderUnsupported {
                        provider: "android".into(),
                        reason: format!(
                            "Mirror '{}' does not support Android repositories",
                            mirror.id
                        ),
                    });
                }
                "ios" => {
                    return Err(PdownError::ProviderUnsupported {
                        provider: "ios".into(),
                        reason: "iOS provider is guarded and unsupported".into(),
                    });
                }
                _ => {}
            }
        }
    }

    let mut plans = Vec::new();

    // 1. Unified Flutter & Dart Environment Plan
    if has_flutter || has_dart {
        let mut vars = Vec::new();
        if has_flutter {
            if let Some(ref f) = mirror.providers.flutter {
                vars.push(("FLUTTER_STORAGE_BASE_URL", f.storage_url.as_str()));
            }
        }
        if has_dart {
            if let Some(ref d) = mirror.providers.dart {
                vars.push(("PUB_HOSTED_URL", d.hosted_url.as_str()));
            }
        }

        let mut items = Vec::new();
        for file in crate::system::environment::EnvironmentManager::get_unix_shell_files() {
            items.push(
                crate::system::environment::EnvironmentManager::plan_unix_file_update(
                    &file,
                    crate::system::environment::MARKER_START_FLUTTER_DART,
                    crate::system::environment::MARKER_END_FLUTTER_DART,
                    &vars,
                )?,
            );
        }

        let mut desc_parts = Vec::new();
        if has_flutter {
            desc_parts.push("Flutter");
        }
        if has_dart {
            desc_parts.push("Dart");
        }

        plans.push(crate::core::models::ChangePlan {
            provider_id: desc_parts.join(" & "),
            description: format!(
                "Configure environment variables for {}",
                desc_parts.join(" and ")
            ),
            items,
        });
    }

    // 2. Gradle Plan (executed once for gradle and/or android)
    if has_gradle || has_android {
        let mut gradle_plan = crate::providers::GradleProvider.plan_apply(&mirror)?;
        if has_gradle && has_android {
            gradle_plan.provider_id = "Gradle & Android".into();
        }
        plans.push(gradle_plan);
    }

    if plans.is_empty() {
        return Err(PdownError::InvalidConfig(format!(
            "No eligible providers found to activate for mirror '{}'",
            mirror.id
        )));
    }

    let backup_mgr = crate::system::backup::BackupManager::new(storage);
    let engine = crate::core::transaction::TransactionEngine::new(backup_mgr, args.dry_run);
    let report = engine.execute(plans)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    if args.dry_run {
        println!(
            "\n🔍 [DRY-RUN] Proposed Changes for mirror '{}':\n",
            mirror.name
        );
        for diff in &report.diffs {
            println!("{diff}\n");
        }
        println!("ℹ️ Dry-run completed. No files were modified.");
    } else {
        println!(
            "\n✅ Successfully activated mirror: {} [{}]",
            mirror.name, mirror.id
        );
        println!(
            "   Providers configured: {}",
            report.successful_providers.join(", ")
        );
        println!("   Files modified:       {}", report.modified_files.len());
        println!("   Backups created:      {}", report.backups.len());
        println!("\n📌 Reminder: Please restart your terminal or IDE (VS Code / Android Studio) for changes to take effect.");
    }

    Ok(())
}

async fn handle_reset(
    storage: &StorageManager,
    args: crate::app::cli::ResetArgs,
    json: bool,
) -> Result<()> {
    let only_set: Option<Vec<String>> = args
        .only
        .map(|s| s.split(',').map(|p| p.trim().to_lowercase()).collect());

    let mut plans = Vec::new();
    let providers = crate::providers::all_providers();

    for p in providers {
        let p_id = p.id().to_lowercase();
        if let Some(ref filter) = only_set {
            if !filter.contains(&p_id) {
                continue;
            }
        }
        plans.push(p.plan_revert()?);
    }

    let backup_mgr = crate::system::backup::BackupManager::new(storage);
    let engine = crate::core::transaction::TransactionEngine::new(backup_mgr, args.dry_run);
    let report = engine.execute(plans)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    if args.dry_run {
        println!("\n🔍 [DRY-RUN] Proposed Reset Operations:\n");
        for diff in &report.diffs {
            println!("{diff}\n");
        }
        println!("ℹ️ Dry-run completed. No files were modified.");
    } else {
        println!("\n🔄 Successfully reset configuration to official Google & upstream defaults.");
        println!(
            "   Providers reset: {}",
            report.successful_providers.join(", ")
        );
        println!("   Files updated:   {}", report.modified_files.len());
    }

    Ok(())
}

async fn handle_test(
    registry: &RegistryManager,
    args: crate::app::cli::TestArgs,
    json: bool,
) -> Result<()> {
    let engine = crate::core::benchmark::BenchmarkEngine::new(args.timeout);

    let mirrors = match args.mirror {
        Some(id) => {
            let m = registry
                .find_mirror(&id)?
                .ok_or_else(|| PdownError::MirrorNotFound(id))?;
            vec![m]
        }
        None => registry.list_mirrors(None)?,
    };

    if !json {
        println!(
            "\n⚡ Testing Mirror Reachability & Latency (timeout: {}s)...\n",
            args.timeout
        );
    }

    let summaries = engine.benchmark_all(&mirrors).await;

    if json {
        println!("{}", serde_json::to_string_pretty(&summaries)?);
        return Ok(());
    }

    for s in &summaries {
        let badge = if s.overall_reachable {
            format!("✅ ONLINE (avg: {}ms)", s.average_latency_ms)
        } else {
            "❌ UNREACHABLE".to_string()
        };

        println!("🔹 [{}] {} — {}", s.mirror_id, s.mirror_name, badge);
        for p in &s.probes {
            let p_badge = if p.reachable {
                format!(
                    "{}ms (HTTP {})",
                    p.total_latency_ms,
                    p.http_status.unwrap_or(200)
                )
            } else {
                format!("FAILED ({})", p.error.as_deref().unwrap_or("unknown error"))
            };
            println!("   • {:<24} -> {}", p.provider_id, p_badge);
        }
        println!();
    }

    if let Some(best) = summaries.iter().find(|s| s.overall_reachable) {
        println!(
            "💡 Recommended Mirror: [{}] {} (Average Latency: {}ms)",
            best.mirror_id, best.mirror_name, best.average_latency_ms
        );
        println!(
            "   To activate this mirror, run: pdown use {}\n",
            best.mirror_id
        );
    }

    Ok(())
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
