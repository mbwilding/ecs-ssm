use anyhow::{bail, Context, Result};
use dialoguer::theme::ColorfulTheme;
use dialoguer::FuzzySelect;

pub async fn get_cluster(
    ecs: &aws_sdk_ecs::Client,
    needle: &Option<String>,
) -> Result<String> {
    let clusters = ecs
        .list_clusters()
        .send()
        .await?
        .cluster_arns
        .context("Could not list clusters")?;

    find_item("Cluster", needle, clusters)
}

pub async fn get_service(
    ecs: &aws_sdk_ecs::Client,
    needle: &Option<String>,
    cluster: &str,
) -> Result<String> {
    let services = ecs
        .list_services()
        .cluster(cluster)
        .send()
        .await?
        .service_arns
        .context("Could not list services")?;

    find_item("Service", needle, services)
}

pub async fn get_task(
    ecs: &aws_sdk_ecs::Client,
    needle: &Option<String>,
    cluster: &str,
    service: &str,
) -> Result<String> {
    let tasks = ecs
        .list_tasks()
        .cluster(cluster)
        .service_name(service)
        .send()
        .await?
        .task_arns
        .context("Could not list tasks")?;

    find_item("Task", needle, tasks)
}

/// This works for FarGate only
pub async fn get_task_os(
    ecs: &aws_sdk_ecs::Client,
    cluster: &str,
    task_arn: &str,
) -> Result<String> {
    let response = ecs
        .describe_tasks()
        .cluster(cluster)
        .tasks(task_arn)
        .send()
        .await?;

    if let Some(task) = response.tasks().first() {
        if let Some(platform_family) = &task.platform_family {
            return Ok(platform_family.to_string());
        }
    }

    bail!("No task data found");
}

pub fn select_item(prompt: &str, items: &[String]) -> Result<String> {
    if items.len() == 1 {
        Ok(items[0].clone())
    } else {
        let selected_index = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(items)
            .interact()?;

        Ok(items[selected_index].clone())
    }
}

pub fn find_item(
    name: &str,
    needle: &Option<String>,
    items: Vec<String>,
) -> Result<String> {
    if let Some(value) = needle {
        items
            .into_iter()
            .find(|c| c.contains(value))
            .with_context(|| format!("Could not find {}: {}", name.to_lowercase(), value))
    } else {
        select_item(name, &items)
            .with_context(|| format!("Error selecting {}s", name.to_lowercase()))
    }
}
