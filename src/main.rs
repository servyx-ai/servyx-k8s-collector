use anyhow::Result;
use clap::Parser;
use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, StatefulSet};
use k8s_openapi::api::core::v1::{Namespace, Node, Pod};
use kube::{Api, Client};
use serde::Serialize;

#[derive(Parser)]
#[command(name = "servyx-k8s-collector", about = "Servyx Kubernetes infrastructure collector")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Collect Kubernetes infrastructure data and send to Servyx
    Collect {
        /// Cluster identifier
        #[arg(long)]
        cluster_id: String,

        /// Servyx platform URL
        #[arg(long)]
        servyx_url: String,

        /// Collector token for authentication
        #[arg(long)]
        token: String,
    },
}

#[derive(Serialize)]
struct K8sSnapshot {
    collector_version: String,
    source_type: String,
    cluster_identifier: String,
    collected_at: String,
    data: K8sData,
}

#[derive(Serialize)]
struct K8sData {
    namespaces: Vec<serde_json::Value>,
    nodes: Vec<serde_json::Value>,
    pods: Vec<serde_json::Value>,
    deployments: Vec<serde_json::Value>,
    statefulsets: Vec<serde_json::Value>,
    daemonsets: Vec<serde_json::Value>,
    metrics: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Collect {
            cluster_id,
            servyx_url,
            token,
        } => {
            println!("Servyx K8s Collector v0.1.0");
            println!("Cluster: {}", cluster_id);

            let client = Client::try_default().await?;

            // Collect namespaces
            println!("Collecting namespaces...");
            let ns_api: Api<Namespace> = Api::all(client.clone());
            let ns_list = ns_api.list(&Default::default()).await?;
            let namespaces: Vec<serde_json::Value> = ns_list
                .items
                .iter()
                .map(|ns| {
                    serde_json::json!({
                        "name": ns.metadata.name.as_deref().unwrap_or(""),
                        "status": ns.status.as_ref().and_then(|s| s.phase.as_deref()).unwrap_or(""),
                        "labels": ns.metadata.labels,
                    })
                })
                .collect();
            println!("  Found {} namespaces", namespaces.len());

            // Collect nodes
            println!("Collecting nodes...");
            let node_api: Api<Node> = Api::all(client.clone());
            let node_list = node_api.list(&Default::default()).await?;
            let nodes: Vec<serde_json::Value> = node_list
                .items
                .iter()
                .map(|node| {
                    let capacity = node.status.as_ref().and_then(|s| s.capacity.as_ref());
                    let allocatable = node.status.as_ref().and_then(|s| s.allocatable.as_ref());
                    serde_json::json!({
                        "name": node.metadata.name.as_deref().unwrap_or(""),
                        "labels": node.metadata.labels,
                        "capacity": {
                            "cpu": capacity.and_then(|c| c.get("cpu")).map(|v| v.0.clone()),
                            "memory": capacity.and_then(|c| c.get("memory")).map(|v| v.0.clone()),
                        },
                        "allocatable": {
                            "cpu": allocatable.and_then(|a| a.get("cpu")).map(|v| v.0.clone()),
                            "memory": allocatable.and_then(|a| a.get("memory")).map(|v| v.0.clone()),
                        },
                    })
                })
                .collect();
            println!("  Found {} nodes", nodes.len());

            // Collect pods
            println!("Collecting pods...");
            let pod_api: Api<Pod> = Api::all(client.clone());
            let pod_list = pod_api.list(&Default::default()).await?;
            let pods: Vec<serde_json::Value> = pod_list
                .items
                .iter()
                .map(|pod| {
                    let containers: Vec<serde_json::Value> = pod
                        .spec
                        .as_ref()
                        .map(|s| {
                            s.containers
                                .iter()
                                .map(|c| {
                                    let resources = c.resources.as_ref();
                                    serde_json::json!({
                                        "name": c.name,
                                        "image": c.image,
                                        "requests": resources.and_then(|r| r.requests.as_ref()).map(|r| {
                                            serde_json::json!({
                                                "cpu": r.get("cpu").map(|v| v.0.clone()),
                                                "memory": r.get("memory").map(|v| v.0.clone()),
                                            })
                                        }),
                                        "limits": resources.and_then(|r| r.limits.as_ref()).map(|l| {
                                            serde_json::json!({
                                                "cpu": l.get("cpu").map(|v| v.0.clone()),
                                                "memory": l.get("memory").map(|v| v.0.clone()),
                                            })
                                        }),
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    serde_json::json!({
                        "name": pod.metadata.name.as_deref().unwrap_or(""),
                        "namespace": pod.metadata.namespace.as_deref().unwrap_or(""),
                        "status": pod.status.as_ref().and_then(|s| s.phase.as_deref()).unwrap_or(""),
                        "node_name": pod.spec.as_ref().and_then(|s| s.node_name.as_deref()).unwrap_or(""),
                        "containers": containers,
                    })
                })
                .collect();
            println!("  Found {} pods", pods.len());

            // Collect deployments
            println!("Collecting deployments...");
            let deploy_api: Api<Deployment> = Api::all(client.clone());
            let deploy_list = deploy_api.list(&Default::default()).await?;
            let deployments: Vec<serde_json::Value> = deploy_list
                .items
                .iter()
                .map(|d| {
                    serde_json::json!({
                        "name": d.metadata.name.as_deref().unwrap_or(""),
                        "namespace": d.metadata.namespace.as_deref().unwrap_or(""),
                        "replicas": d.spec.as_ref().and_then(|s| s.replicas),
                        "ready_replicas": d.status.as_ref().and_then(|s| s.ready_replicas),
                        "available_replicas": d.status.as_ref().and_then(|s| s.available_replicas),
                    })
                })
                .collect();
            println!("  Found {} deployments", deployments.len());

            // Collect statefulsets
            println!("Collecting statefulsets...");
            let ss_api: Api<StatefulSet> = Api::all(client.clone());
            let ss_list = ss_api.list(&Default::default()).await?;
            let statefulsets: Vec<serde_json::Value> = ss_list
                .items
                .iter()
                .map(|s| {
                    serde_json::json!({
                        "name": s.metadata.name.as_deref().unwrap_or(""),
                        "namespace": s.metadata.namespace.as_deref().unwrap_or(""),
                        "replicas": s.spec.as_ref().and_then(|sp| sp.replicas),
                        "ready_replicas": s.status.as_ref().map(|st| st.ready_replicas),
                    })
                })
                .collect();
            println!("  Found {} statefulsets", statefulsets.len());

            // Collect daemonsets
            println!("Collecting daemonsets...");
            let ds_api: Api<DaemonSet> = Api::all(client.clone());
            let ds_list = ds_api.list(&Default::default()).await?;
            let daemonsets: Vec<serde_json::Value> = ds_list
                .items
                .iter()
                .map(|d| {
                    serde_json::json!({
                        "name": d.metadata.name.as_deref().unwrap_or(""),
                        "namespace": d.metadata.namespace.as_deref().unwrap_or(""),
                        "desired": d.status.as_ref().map(|s| s.desired_number_scheduled),
                        "ready": d.status.as_ref().map(|s| s.number_ready),
                    })
                })
                .collect();
            println!("  Found {} daemonsets", daemonsets.len());

            let snapshot = K8sSnapshot {
                collector_version: "0.1.0".to_string(),
                source_type: "kubernetes".to_string(),
                cluster_identifier: cluster_id,
                collected_at: chrono::Utc::now().to_rfc3339(),
                data: K8sData {
                    namespaces,
                    nodes,
                    pods,
                    deployments,
                    statefulsets,
                    daemonsets,
                    metrics: serde_json::json!({}),
                },
            };

            println!("\nSending to {}...", servyx_url);
            let http = reqwest::Client::new();
            let res = http
                .post(format!("{}/api/ingest/kubernetes", servyx_url))
                .header("Authorization", format!("Bearer {}", token))
                .json(&snapshot)
                .send()
                .await?;

            if res.status().is_success() {
                println!("Successfully sent snapshot to Servyx!");
            } else {
                let status = res.status();
                let body = res.text().await.unwrap_or_default();
                eprintln!("Ingestion failed ({}): {}", status, body);
                std::process::exit(1);
            }

            println!("Collection complete.");
        }
    }

    Ok(())
}
