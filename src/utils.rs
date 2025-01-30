use reqwest::{header, Client};
use std::{process::Command, time::Duration};
use tokio::time::sleep;

use crate::models::google::{
    Instance, InstancesResponse, IpConfiguration, Operation, OperationStatus, Project,
    ProjectsResponse, Settings,
};
use crate::models::{InstanceUpdateDto, NetworkUpdateDto};
use crate::{error, GError, GResult};

pub async fn auth_token() -> String {
    let gcloud_auth_cmd = Command::new("gcloud")
        .arg("auth")
        .arg("print-access-token")
        .output()
        .unwrap_or_else(|_| {
            error!("Unable get auth token, please make sure gcloud CLI is installed and you're logged in.");
            std::process::exit(1);
        });
    let access_token = String::from_utf8(gcloud_auth_cmd.stdout).unwrap();

    return access_token.trim().to_string();
}

pub async fn current_ip_cidr_notation() -> String {
    let client = Client::new();
    let checkip_response = client
        .get("https://checkip.amazonaws.com")
        .send()
        .await
        .expect("Failed to get IP address")
        .text()
        .await
        .expect("Failed to parse IP address response");
    let current_ip = checkip_response.trim();
    let current_ip_octets: Vec<&str> = current_ip.split(".").collect();
    let current_ip_cidr_notation = format!(
        "{}.{}.{}.0/24",
        current_ip_octets[0], current_ip_octets[1], current_ip_octets[2]
    );

    return current_ip_cidr_notation;
}

pub async fn fetch_projects() -> Vec<Project> {
    let access_token = auth_token().await;

    let client = Client::new();
    let projects_response = client
        .get("https://cloudresourcemanager.googleapis.com/v1/projects")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .query(&[("filter", "lifecycleState:ACTIVE parent.type:organization")])
        .send()
        .await
        .expect("Failed to send request to the API")
        .text()
        .await
        .expect("Failed to read API response body as text");
    let ProjectsResponse { projects } = serde_json::from_str(&projects_response).expect(&format!(
        "Failed to parse API response as JSON: {}",
        projects_response
    ));

    return projects;
}

pub async fn fetch_instances(project_id: &str) -> Vec<Instance> {
    let access_token = auth_token().await;

    let client = Client::new();
    let instances_response = client
        .get(format!(
            "https://sqladmin.googleapis.com/v1/projects/{}/instances",
            project_id
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .query(&[("filter", "state:RUNNABLE instanceType:CLOUD_SQL_INSTANCE")])
        .send()
        .await
        .expect("Failed to send request to the API")
        .text()
        .await
        .expect("Failed to read API response body as text");
    let InstancesResponse { items } = serde_json::from_str(&instances_response).expect(&format!(
        "Failed to parse API response as JSON: {}",
        instances_response
    ));

    return items;
}

pub async fn fetch_instance(project_id: &str, instance_id: &str) -> GResult<Instance> {
    let access_token = auth_token().await;

    let client = Client::new();
    let instances_response = client
        .get(format!(
            "https://sqladmin.googleapis.com/v1/projects/{}/instances/{}",
            project_id, instance_id
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .send()
        .await
        .expect("Failed to send request to the API")
        .text()
        .await
        .expect("Failed to read API response body as text");
    let instance: Instance = serde_json::from_str(&instances_response).expect(&format!(
        "Failed to parse API response as JSON: {}",
        instances_response
    ));

    return Ok(instance);
}

pub async fn update_instance_network(
    instance: &Instance,
    network_update_dto: NetworkUpdateDto,
) -> Operation {
    let access_token = auth_token().await;
    let mut authorized_networks = instance
        .settings
        .ip_configuration
        .authorized_networks
        .clone();
    let network_index = authorized_networks
        .iter()
        .position(|network| network.name == network_update_dto.name)
        .unwrap_or_else(|| {
            error!(
                "Unable to find the network `{}`, instance[{}] project[{}]",
                network_update_dto.name, instance.name, instance.project
            );
            std::process::exit(1);
        });
    authorized_networks[network_index].value = network_update_dto.value;

    let instance_update_dto = InstanceUpdateDto {
        settings: Settings {
            ip_configuration: IpConfiguration {
                authorized_networks,
            },
        },
    };

    let client = Client::new();
    let instance_update_response = client
        .patch(format!(
            "https://sqladmin.googleapis.com/v1/projects/{}/instances/{}",
            instance.project, instance.name
        ))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .body(
            serde_json::to_string(&instance_update_dto)
                .expect("Failed to serialize the instance update DTO"),
        )
        .send()
        .await
        .expect("Failed to send request to the API")
        .text()
        .await
        .expect("Failed to read API response body as text");
    let operation: Operation = serde_json::from_str(&instance_update_response).expect(&format!(
        "Failed to parse API response as JSON: {}",
        instance_update_response
    ));

    return operation;
}

pub async fn operation_status(project_id: &str, operation_id: &str) -> GResult<()> {
    let mut attempt = 0;
    let max_attempts = 15;
    let access_token = auth_token().await;

    let client = Client::new();

    loop {
        let operation_response = client
            .get(format!(
                "https://sqladmin.googleapis.com/v1/projects/{}/operations/{}",
                project_id, operation_id,
            ))
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
            .send()
            .await
            .expect("Failed to send request to the API")
            .text()
            .await
            .expect("Failed to read API response body as text");
        let operation: Operation = serde_json::from_str(&operation_response).expect(&format!(
            "Failed to parse API response as JSON: {}",
            operation_response
        ));

        match operation.status {
            OperationStatus::Done => {
                return Ok(());
            }
            OperationStatus::Unspecified => {
                return Err(GError::Unknown("Unknown status".into()));
            }
            OperationStatus::Pending | OperationStatus::Running => {}
        }

        attempt += 1;

        if attempt > max_attempts {
            return Err(GError::Timeout("Max retries reached".into()));
        }

        let wait_time = Duration::from_secs(2_u64.pow(attempt));
        sleep(wait_time).await;
    }
}
