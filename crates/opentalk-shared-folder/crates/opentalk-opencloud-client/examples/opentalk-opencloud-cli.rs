// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use clap::{ArgAction, Args, Parser, Subcommand};
use opentalk_opencloud_client::{
    Client, CreateLinkOptions, DriveId, Error, ItemId, PermissionId, SharingLinkType,
};
use url::Url;

#[derive(Args)]
struct OpenCloudParameters {
    #[arg(env = "OPENCLOUD_BASE_URL", long)]
    base_url: Url,

    #[arg(env = "OPENCLOUD_USERNAME", long)]
    username: String,

    #[arg(env = "OPENCLOUD_PASSWORD", long)]
    password: String,
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List the drives (spaces) available to the user
    ListDrives {
        #[command(flatten)]
        opencloud: OpenCloudParameters,
    },
    /// Print the share-link password policy reported by the server
    PasswordPolicy {
        #[command(flatten)]
        opencloud: OpenCloudParameters,
    },
    /// Generate a password that conforms to the server's password policy
    GeneratePassword {
        #[command(flatten)]
        opencloud: OpenCloudParameters,
    },
    /// Create a folder in a drive on the OpenCloud instance
    CreateFolder {
        /// Identifier of the drive the folder should be created in
        drive_id: DriveId,

        /// Path of the folder to create
        path: String,

        #[command(flatten)]
        opencloud: OpenCloudParameters,
    },
    /// Delete a folder from a drive on the OpenCloud instance
    DeleteFolder {
        /// Identifier of the drive the folder is located in
        drive_id: DriveId,

        /// Path of the folder to delete
        path: String,

        #[command(flatten)]
        opencloud: OpenCloudParameters,
    },
    /// Create a share link for an existing folder on the OpenCloud instance
    CreateLink {
        /// Identifier of the drive the folder is located in
        drive_id: DriveId,

        /// Path of the folder that should be shared
        path: String,

        #[command(flatten)]
        opencloud: OpenCloudParameters,

        /// Type of the link
        #[arg(long)]
        link_type: Option<SharingLinkType>,

        /// Password for the link
        #[arg(long = "link-password", id = "link_password")]
        password: Option<String>,

        /// Expiration date and time for the link (RFC 3339, e.g. 2026-12-31T23:59:59Z)
        #[arg(long)]
        expiration: Option<DateTime<Utc>>,

        /// Display name for the link
        #[arg(long)]
        display_name: Option<String>,

        /// Mark the link as a quick link
        #[arg(long, action = ArgAction::SetTrue)]
        quick_link: bool,
    },
    /// Update a share link on the OpenCloud instance
    UpdateLink {
        /// Identifier of the drive the item is located in
        drive_id: DriveId,

        /// Identifier of the shared item
        item_id: ItemId,

        /// Identifier of the link permission to update
        permission_id: PermissionId,

        #[command(flatten)]
        opencloud: OpenCloudParameters,

        /// New type for the link
        #[arg(long)]
        link_type: Option<SharingLinkType>,

        /// New expiration date and time (RFC 3339)
        #[arg(long)]
        expiration: Option<DateTime<Utc>>,

        /// Remove the expiration from the link
        #[arg(long, action = ArgAction::SetTrue)]
        remove_expiration: bool,

        /// New password for the link
        #[arg(long = "link-password", id = "link_password")]
        password: Option<String>,
    },
    /// Delete a share link from the OpenCloud instance
    DeleteLink {
        /// Identifier of the drive the item is located in
        drive_id: DriveId,

        /// Identifier of the shared item
        item_id: ItemId,

        /// Identifier of the link permission to delete
        permission_id: PermissionId,

        #[command(flatten)]
        opencloud: OpenCloudParameters,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::ListDrives { opencloud } => {
            let client = Client::new(opencloud.base_url, opencloud.username, opencloud.password)?;
            for drive in client.get_drives().await? {
                println!("{}\t{}\t{}", drive.id, drive.drive_type, drive.name);
            }
        }
        Commands::PasswordPolicy { opencloud } => {
            let client = Client::new(opencloud.base_url, opencloud.username, opencloud.password)?;
            let policy = client.get_password_policy().await?;
            println!("{policy:#?}");
        }
        Commands::GeneratePassword { opencloud } => {
            let client = Client::new(opencloud.base_url, opencloud.username, opencloud.password)?;
            println!("{}", client.generate_password().await?);
        }
        Commands::CreateFolder {
            drive_id,
            path,
            opencloud,
        } => {
            let client = Client::new(opencloud.base_url, opencloud.username, opencloud.password)?;
            client.create_folder(&drive_id, &path).await?;
            println!("Created folder {path}");
        }
        Commands::DeleteFolder {
            drive_id,
            path,
            opencloud,
        } => {
            let client = Client::new(opencloud.base_url, opencloud.username, opencloud.password)?;
            client.delete_folder(&drive_id, &path).await?;
            println!("Deleted folder {path}");
        }
        Commands::CreateLink {
            drive_id,
            path,
            opencloud,
            link_type,
            password,
            expiration,
            display_name,
            quick_link,
        } => {
            let client = Client::new(opencloud.base_url, opencloud.username, opencloud.password)?;
            let options = CreateLinkOptions {
                link_type: link_type.unwrap_or_default(),
                expiration,
                password,
                display_name,
                quick_link,
            };
            let created = client.create_link(&drive_id, &path, options).await?;
            let web_url = created
                .permission
                .link
                .as_ref()
                .map(|link| link.web_url.as_str())
                .unwrap_or_default();
            println!(
                "Created link {} for item {} at {}",
                created.permission.id, created.item_id, web_url
            );
        }
        Commands::UpdateLink {
            drive_id,
            item_id,
            permission_id,
            opencloud,
            link_type,
            expiration,
            remove_expiration,
            password,
        } => {
            let client = Client::new(opencloud.base_url, opencloud.username, opencloud.password)?;
            let request = client.update_link(drive_id, item_id, permission_id);

            if let Some(v) = link_type {
                let permission = request.link_type(v).await?;
                println!("Updated type for link {}", permission.id);
            } else if let Some(v) = expiration {
                let permission = request.expiration(Some(v)).await?;
                println!("Updated expiration for link {}", permission.id);
            } else if remove_expiration {
                let permission = request.expiration(None).await?;
                println!("Removed expiration for link {}", permission.id);
            } else if let Some(v) = password {
                let permission = request.password(v).await?;
                println!("Updated password for link {}", permission.id);
            } else {
                println!("No update parameter given");
            }
        }
        Commands::DeleteLink {
            drive_id,
            item_id,
            permission_id,
            opencloud,
        } => {
            let client = Client::new(opencloud.base_url, opencloud.username, opencloud.password)?;
            client
                .delete_link(&drive_id, &item_id, &permission_id)
                .await?;
            println!("Deleted link {permission_id}");
        }
    }

    Ok(())
}
