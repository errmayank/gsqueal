use std::time::Duration;

use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    error, info,
    models::{NetworkOperation, NetworkUpdateDto},
    store::GStore,
    utils, warn,
};

pub async fn update(repeat_last: &bool) {
    if *repeat_last {
        let store = GStore::get().await;

        if let Some(operation) = store.network.last_operation {
            let NetworkOperation {
                project_id,
                instance_id,
                network_name,
            } = operation;

            let instance = match utils::fetch_instance(&project_id, &instance_id).await {
                Ok(instance) => instance,
                Err(_) => {
                    error!(
                        "Cannot fetch instance `{}` under project `{}`",
                        &project_id, &instance_id,
                    );
                    std::process::exit(1);
                }
            };
            let ip = utils::current_ip_cidr_notation().await;
            let network = instance
                .settings
                .ip_configuration
                .authorized_networks
                .iter()
                .find(|network| network.name == network_name)
                .unwrap_or_else(|| {
                    error!(
                        "Unable to find the network `{}`, instance[{}] project[{}]",
                        network_name, instance.name, instance.project
                    );
                    std::process::exit(1);
                });

            if network.value == ip {
                warn!("Skip update, network is already set to your current IP.");
            } else {
                let network_update_dto = NetworkUpdateDto {
                    name: network.name.clone(),
                    value: ip,
                };

                let operation = utils::update_instance_network(&instance, network_update_dto).await;
                let spinner = ProgressBar::new_spinner();
                spinner.set_style(
                    ProgressStyle::default_spinner()
                        .template("{spinner} Processing...")
                        .unwrap(),
                );
                spinner.enable_steady_tick(Duration::from_millis(100));
                let operation_status =
                    utils::operation_status(&instance.project, &operation.name).await;

                spinner.finish_and_clear();

                match operation_status {
                    Ok(()) => {
                        info!("Operation completed successfully! ✅");
                    }
                    Err(message) => {
                        error!("Operation failed: {}", message);
                    }
                }
            }
        } else {
            error!("Cannot find any previous network update operations");
            std::process::exit(1);
        }
    } else {
        let fetching_projects_spinner = ProgressBar::new_spinner();
        fetching_projects_spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner} Fetching projects...")
                .unwrap(),
        );
        fetching_projects_spinner.enable_steady_tick(Duration::from_millis(100));

        let mut projects = utils::fetch_projects().await;
        fetching_projects_spinner.finish_and_clear();

        if projects.is_empty() {
            error!("Cannot find any projects under your account");
            std::process::exit(1);
        }
        projects.sort_by_key(|project| project.name.to_lowercase());

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a project")
            .default(0)
            .max_length(10)
            .items(
                &projects
                    .iter()
                    .map(|project| project.name.as_str())
                    .collect::<Vec<_>>(),
            )
            .interact()
            .unwrap();
        let selected_project = &projects[selection];

        let spinner_template = format!(
            "{{spinner}} Fetching {}'s instances...",
            style(&selected_project.project_id).green(),
        );
        let fetching_instances_spinner = ProgressBar::new_spinner();
        fetching_instances_spinner.set_style(
            ProgressStyle::default_spinner()
                .template(&spinner_template)
                .unwrap(),
        );
        fetching_instances_spinner.enable_steady_tick(Duration::from_millis(100));

        let instances = utils::fetch_instances(&selected_project.project_id).await;
        fetching_instances_spinner.finish_and_clear();

        if instances.is_empty() {
            error!(
                "Cannot find any instances under your account for project `{}`",
                selected_project.project_id
            );
            std::process::exit(1);
        }

        let instance_selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an instance")
            .default(0)
            .max_length(10)
            .items(
                &instances
                    .iter()
                    .map(|instance| instance.name.as_str())
                    .collect::<Vec<_>>(),
            )
            .interact()
            .unwrap();
        let selected_instance = &instances[instance_selection];
        let mut authorized_networks = selected_instance
            .settings
            .ip_configuration
            .authorized_networks
            .clone();
        if authorized_networks.is_empty() {
            error!(
                "Cannot find any authorized networks for instance[{}] project[{}]",
                selected_instance.name, selected_instance.project
            );
            std::process::exit(1);
        }

        authorized_networks.sort_by_key(|network| network.name.to_lowercase());

        let network_selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a network")
            .default(0)
            .max_length(10)
            .items(
                &authorized_networks
                    .iter()
                    .map(|network| network.name.as_str())
                    .collect::<Vec<_>>(),
            )
            .interact()
            .unwrap();
        let selected_network = authorized_networks[network_selection].clone();

        let ip = utils::current_ip_cidr_notation().await;

        if selected_network.value == ip {
            warn!("Skip update, network is already set to your current IP.");
        } else {
            let prompt = format!(
                "Update {} to current IP {}. Continue?",
                style(&selected_network.name).green().bold(),
                style(&ip).green().bold(),
            );
            let confirmation = Confirm::new().with_prompt(prompt).interact().unwrap();

            if confirmation == false {
                warn!("Aborting...");
                std::process::exit(1);
            } else {
                let network_update_dto = NetworkUpdateDto {
                    name: selected_network.name.clone(),
                    value: ip,
                };

                let operation =
                    utils::update_instance_network(selected_instance, network_update_dto).await;

                let spinner = ProgressBar::new_spinner();
                spinner.set_style(
                    ProgressStyle::default_spinner()
                        .template("{spinner} Processing...")
                        .unwrap(),
                );
                spinner.enable_steady_tick(Duration::from_millis(100));
                let operation_status =
                    utils::operation_status(&selected_project.project_id, &operation.name).await;

                spinner.finish_and_clear();

                match operation_status {
                    Ok(()) => {
                        let mut store = GStore::get().await;
                        store.network.last_operation = Some(NetworkOperation {
                            project_id: selected_project.project_id.to_string(),
                            instance_id: selected_instance.name.to_string(),
                            network_name: selected_network.name.to_string(),
                        });
                        GStore::set(&store).await;
                        info!("Operation completed successfully!");
                    }
                    Err(message) => {
                        error!("Operation failed: {}", message);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}
