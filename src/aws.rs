use crate::helpers::*;
use anyhow::Result;
use anyhow::{anyhow, bail, Context, Ok};
use aws_config::{Region, SdkConfig};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::process::{Command, Stdio};

#[derive(Deserialize)]
pub struct AwsCredential {
    azure_tenant_id: Option<String>,
    credential_process: Option<String>,
}

pub fn get_aws_config_sections() -> Result<HashMap<String, AwsCredential>> {
    let home_dir = dirs::home_dir().context("Could not find home directory")?;
    let config_path = home_dir.join(".aws/config");
    let content = fs::read_to_string(config_path)?;
    let new_config = serde_ini::from_str::<HashMap<String, AwsCredential>>(&content)?
        .into_iter()
        .map(|(key, value)| {
            (
                key.strip_prefix("profile ").unwrap_or(&key).to_string(),
                value,
            )
        })
        .collect();

    Ok(new_config)
}

pub async fn aws_login(
    profile: &Option<String>,
    region: &Option<String>,
) -> Result<(String, SdkConfig)> {
    let aws_profiles = get_aws_config_sections()?;
    let mut aws_profile_names: Vec<String> = aws_profiles.keys().cloned().collect();
    aws_profile_names.sort();
    aws_profile_names.retain(|profile| profile != "default");
    aws_profile_names.insert(0, "default".to_string());

    let profile_name = if let Some(profile) = profile {
        profile
    } else {
        &select_item("Account", &aws_profile_names)
            .context("Could not list AWS profiles")?
            .to_string()
    };

    let aws_profile = aws_profiles
        .get(profile_name)
        .ok_or_else(|| anyhow!("AWS profile does not exist: {}", profile_name))?;

    if aws_profile.azure_tenant_id.is_some() && aws_profile.credential_process.is_none() {
        let output = Command::new("aws-azure-login")
            .args(["--profile", profile_name])
            .output()?;

        if !output.status.success() {
            bail!(format!(
                "Failed to log into AWS with profile: {}",
                profile_name
            ));
        }
    }

    let mut aws_config = aws_config::from_env().profile_name(profile_name);
    if let Some(region) = region {
        let region = Region::new(region.to_string());
        aws_config = aws_config.region(region);
    }
    let aws_config = aws_config.load().await;

    Ok((profile_name.into(), aws_config))
}

pub fn interactive_container_shell(
    profile: &str,
    region: Option<String>,
    cluster: &str,
    task: &str,
    entry: Option<String>,
    platform: &str,
) -> Result<()> {
    let entry = if let Some(entry) = entry {
        entry
    } else if platform.to_lowercase().contains("windows") {
        "powershell.exe".to_string()
    } else {
        "/bin/bash".to_string()
    };

    let mut command = Command::new("aws");

    command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .args([
            "ecs",
            "execute-command",
            "--cluster",
            cluster,
            "--task",
            task,
            "--interactive",
            "--command",
            &entry,
            "--profile",
            profile,
        ]);

    if let Some(region) = region {
        command.args(["--region", &region]);
    }

    let _status = command.status()?;

    Ok(())
}
