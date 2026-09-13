// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashSet;

use chrono::NaiveDate;
use clap::{ArgAction, Args, Parser, Subcommand};
use opentalk_nextcloud_client::{Client, Error, ShareId, SharePermission, ShareType};
use url::Url;

#[derive(Args)]
struct NextCloudParameters {
    #[arg(env = "NEXTCLOUD_BASE_URL", long)]
    base_url: Url,

    #[arg(env = "NEXTCLOUD_USERNAME", long)]
    username: String,

    #[arg(env = "NEXTCLOUD_PASSWORD", long)]
    password: String,
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new folder on the NextCloud instance
    CreateFolder {
        path: String,

        #[command(flatten)]
        nextcloud: NextCloudParameters,
    },
    /// Delete a file or folder from the NextCloud instance
    Delete {
        path: String,

        #[command(flatten)]
        nextcloud: NextCloudParameters,
    },
    /// Create a new share for an existing folder on the NextCloud instance
    CreateShare {
        /// Path of the folder that should be shared
        path: String,

        #[command(flatten)]
        nextcloud: NextCloudParameters,

        /// Label for the share
        #[arg(long)]
        label: Option<String>,

        /// Password for the share
        #[arg(long)]
        share_password: Option<String>,

        /// Expire date for the share
        #[arg(long)]
        expire_date: Option<NaiveDate>,

        /// Share permissions
        #[arg(long, value_delimiter = ',')]
        permissions: Option<Vec<SharePermission>>,
    },
    /// Update a share on the NextCloud instance
    UpdateShare {
        /// Path of the folder that should be shared
        id: ShareId,

        #[command(flatten)]
        nextcloud: NextCloudParameters,

        /// Label for the share
        #[arg(long, group("parameter"))]
        label: Option<String>,

        /// Expire date for the share
        #[arg(long, group("parameter"))]
        expire_date: Option<NaiveDate>,

        #[arg(long, group("parameter"), action = ArgAction::SetTrue)]
        remove_expire_date: bool,

        /// Share permissions
        #[arg(long, value_delimiter = ',', group("parameter"))]
        permissions: Option<Vec<SharePermission>>,
    },
    /// Delete a share from the NextCloud instance
    DeleteShare {
        /// Path of the share that should be deleted
        id: ShareId,

        #[command(flatten)]
        nextcloud: NextCloudParameters,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::CreateFolder {
            path,
            nextcloud:
                NextCloudParameters {
                    base_url,
                    username,
                    password,
                },
        } => {
            let client = Client::new(base_url, username.clone(), password)?;
            let path = format!("files/{username}/{path}");
            client.create_folder(&path).await?;
            println!("Created folder {path}");
        }
        Commands::Delete {
            path,
            nextcloud:
                NextCloudParameters {
                    base_url,
                    username,
                    password,
                },
        } => {
            let client = Client::new(base_url, username.clone(), password)?;
            let path = format!("files/{username}/{path}");
            client.delete(&path).await?;
            println!("Deleted {path}");
        }
        Commands::CreateShare {
            path,
            nextcloud:
                NextCloudParameters {
                    base_url,
                    username,
                    password,
                },
            label,
            share_password,
            expire_date,
            permissions,
        } => {
            let client = Client::new(base_url, username.clone(), password)?;
            let path = format!("/{path}");
            let mut request = client.create_share(&path, ShareType::PublicLink);
            if let Some(v) = label {
                request = request.label(v);
            }
            if let Some(v) = share_password {
                request = request.password(v);
            }
            if let Some(v) = expire_date {
                request = request.expire_date(v);
            }
            for v in permissions.into_iter().flatten() {
                request = request.permission(v);
            }
            let response = request.send().await?;
            println!(
                "Created share {} at {} for path {}",
                response.data.id, response.data.url, response.data.file_target
            );
        }
        Commands::UpdateShare {
            id,
            nextcloud:
                NextCloudParameters {
                    base_url,
                    username,
                    password,
                },
            label,
            expire_date,
            remove_expire_date,
            permissions,
        } => {
            let client = Client::new(base_url, username.clone(), password)?;
            let request = client.update_share(id);

            if let Some(v) = label {
                let response = request.label(v).await?;
                println!(
                    "Updated label for share {} at {} for path {}",
                    response.data.id, response.data.url, response.data.file_target
                );
            } else if let Some(v) = expire_date {
                let response = request.expire_date(Some(v)).await?;
                println!(
                    "Updated expire date for share {} at {} for path {}",
                    response.data.id, response.data.url, response.data.file_target
                );
            } else if remove_expire_date {
                let response = request.expire_date(None).await?;
                println!(
                    "Removed expire date for share {} at {} for path {}",
                    response.data.id, response.data.url, response.data.file_target
                );
            } else if let Some(v) = permissions {
                let response = request.permissions(HashSet::from_iter(v)).await?;
                println!(
                    "Updated permissions for share {} at {} for path {}",
                    response.data.id, response.data.url, response.data.file_target
                );
            }
        }
        Commands::DeleteShare {
            id,
            nextcloud:
                NextCloudParameters {
                    base_url,
                    username,
                    password,
                },
        } => {
            let client = Client::new(base_url, username.clone(), password)?;
            client.delete_share(id.clone()).await?;
            println!("Deleted share {id}");
        }
    }

    Ok(())
}
