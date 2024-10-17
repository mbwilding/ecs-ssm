mod aws;
mod helpers;

use anyhow::Result;
use aws::*;
use clap::Parser;
use helpers::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The AWS profile name
    #[arg(short, long)]
    profile: Option<String>,

    /// The AWS region
    #[arg(short, long)]
    region: Option<String>,

    /// The partial cluster name or ARN
    #[arg(short, long)]
    cluster: Option<String>,

    /// The partial service name or ARN
    #[arg(short, long)]
    service: Option<String>,

    /// The partial task name or ARN
    #[arg(short, long)]
    task: Option<String>,

    /// The entry point shell path inside the container
    #[arg(short, long)]
    entry: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let (aws_profile, aws_config) = aws_login(&cli.profile, &cli.region).await?;
    let ecs = aws_sdk_ecs::Client::new(&aws_config);
    let cluster = get_cluster(&ecs, &cli.cluster).await?;
    let service = get_service(&ecs, &cli.service, &cluster).await?;
    let task = get_task(&ecs, &cli.task, &cluster, &service).await?;
    let platform = get_task_os(&ecs, &cluster, &task).await?;
    interactive_container_shell(&aws_profile, cli.region, &cluster, &task, cli.entry, &platform)
}
