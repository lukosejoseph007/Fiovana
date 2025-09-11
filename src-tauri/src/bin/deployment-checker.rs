//! Deployment checker binary for validating release artifacts and deployment readiness
//!
//! Usage:
//!   cargo run --bin deployment-checker -- --manifest      # Generate deployment manifest
//!   cargo run --bin deployment-checker -- --validate-artifacts <artifacts_path> <checksums_path>
//!   cargo run --bin deployment-checker -- --check-readiness  # Full deployment readiness check

use clap::{Arg, Command};
use proxemic::filesystem::security::deployment_checker::DeploymentChecker;
use proxemic::filesystem::security::security_config::SecurityConfigError;
use std::path::PathBuf;

fn main() -> Result<(), SecurityConfigError> {
    let matches = Command::new("deployment-checker")
        .version("1.0.0")
        .about("Validates deployment readiness and release artifacts")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("manifest")
                .about("Generate deployment manifest for artifacts")
                .arg(
                    Arg::new("artifacts_path")
                        .help("Path to artifacts directory")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("validate-artifacts")
                .about("Validate artifact integrity using checksums")
                .arg(
                    Arg::new("artifacts_path")
                        .help("Path to artifacts directory")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("checksums_path")
                        .help("Path to checksums file")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            Command::new("check-readiness")
                .about("Perform comprehensive deployment readiness check"),
        )
        .get_matches();

    let checker = DeploymentChecker::new();

    match matches.subcommand() {
        Some(("manifest", sub_matches)) => {
            let artifacts_path = PathBuf::from(
                sub_matches
                    .get_one::<String>("artifacts_path")
                    .expect("artifacts_path is required"),
            );

            let manifest = checker.generate_deployment_manifest(&artifacts_path)?;
            println!("{}", manifest);
        }
        Some(("validate-artifacts", sub_matches)) => {
            let artifacts_path = PathBuf::from(
                sub_matches
                    .get_one::<String>("artifacts_path")
                    .expect("artifacts_path is required"),
            );
            let checksums_path = PathBuf::from(
                sub_matches
                    .get_one::<String>("checksums_path")
                    .expect("checksums_path is required"),
            );

            checker.validate_release_artifacts(&artifacts_path, &checksums_path)?;
            println!("✅ All artifacts validated successfully");

            checker.verify_code_signing(&artifacts_path)?;
            println!("✅ Code signing verification passed");
        }
        Some(("check-readiness", _)) => {
            let report = checker.generate_deployment_report()?;
            println!("{}", report);

            let assessment = checker.assess_deployment_readiness()?;
            if assessment.ready_for_production {
                println!("🚀 Deployment is READY for production");
                std::process::exit(0);
            } else {
                println!("❌ Deployment is NOT READY for production");
                std::process::exit(1);
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
