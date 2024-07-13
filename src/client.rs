use std::fs::File;

use clap::ArgMatches;
use pterodactyl_api::client::{Client, ClientBuilder, PowerSignal};
use pterodactyl_api::client::backups::BackupParams;
use reqwest::blocking as http;
use tabled::builder::Builder;
use tabled::settings::Style;
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::RemotePath;

pub fn handle_client(matches: &ArgMatches) {

    let panel_url = matches.get_one::<String>("panel-url").unwrap();
    let client_key =  matches.get_one::<String>("client-key").unwrap();

    // Create the Pterodactyl client
    let client = ClientBuilder::new(panel_url, client_key).build();

    match matches.subcommand() {
        Some(("powersignal", sub_m)) => {
            handle_power_signal(&client, sub_m)
        },
        Some(("sendcommand", sub_m)) => {
            handle_send_command(&client, sub_m)
        },
        Some(("upload", sub_m)) => {
            handle_upload(&client, sub_m)
        },
        Some(("download", sub_m)) => {
            handle_download(&client, sub_m)
        },
        Some(("backup", sub_m)) => {
            handle_backup(&client, sub_m)
        },
        Some(("rm", sub_m)) => {
            handle_rm(&client, sub_m)
        },
        _ => {
            // This case should never match when using clap cli
            panic!("No client command found. This should not happen, please report")
        }
    }
}

fn handle_backup(client: &Client, matches: &ArgMatches) {
    match matches.subcommand() {
        Some(("create", sub_m)) => {
            handle_backup_create(&client, sub_m)
        },
        Some(("ls", sub_m)) => {
            handle_backup_ls(&client, sub_m)
        },
        Some(("rm", sub_m)) => {
            handle_backup_rm(&client, sub_m)
        },
        _ => {
            // This case should never match when using clap cli
            panic!("No backup subcommand found. This should not happen, please report")
        }
    }
}

fn handle_backup_ls(client: &Client, matches: &ArgMatches) {
    let servers = matches.get_many::<String>("servers")
        .unwrap_or_default()
        .map(|v| v.as_str())
        .collect::<Vec<_>>();

    for server in servers {
        let result = Runtime::new().unwrap().block_on(client.get_server(server).list_backups());
        match result {
            Ok(backups) => {
                let mut output_builder = Builder::new();
                output_builder.push_record(["Backup name", "Server UID", "Backup UUID", "Created  at", "Is locked?"]);
                for backup in backups {
                    output_builder.push_record([
                        backup.name,
                        server.parse().unwrap(),
                        backup.uuid.to_string(),
                        backup.created_at.to_string(),
                        backup.is_locked.to_string()
                    ]);
                }
                let output = output_builder.build().with(Style::ascii_rounded()).to_string();
                println!("{output}");
            },
            Err(e) => panic!("Error when attempting to get list of backups: {}", e)
        }
    }
}

fn handle_backup_rm(client: &Client, matches: &ArgMatches) {
    let servers = matches.get_many::<String>("servers")
        .unwrap_or_default()
        .map(|v| v.as_str())
        .collect::<Vec<_>>();

    let backup = Uuid::try_parse(matches.get_one::<String>("BACKUP_UUID").unwrap())
        .expect("Error occured when parsing backup UUID");

    for server in servers {
        Runtime::new().unwrap()
            .block_on(client.get_server(server).delete_backup(backup))
            .expect("Error occurred when deleting a backup");
    }

}

fn handle_backup_create(client: &Client, matches: &ArgMatches) {
    let servers = matches.get_many::<String>("servers")
        .unwrap_or_default()
        .map(|v| v.as_str())
        .collect::<Vec<_>>();
    let backup_name = matches.get_one::<String>("name");
    let backup_should_lock = matches.get_flag("lock");

    let mut backup_params = BackupParams::new();

    if backup_name.is_some() {
        backup_params = backup_params.with_name(backup_name.unwrap())
    }
    if backup_should_lock {
        backup_params = backup_params.set_locked();
    }


    for server in servers {
        Runtime::new().unwrap()
            .block_on(client.get_server(server).create_backup_with_params(backup_params.to_owned()))
            .expect("Error while creating backup");
    }
}

fn handle_rm(client: &Client, matches: &ArgMatches) {
    let servers = matches.get_many::<String>("servers")
        .unwrap_or_default()
        .map(|v| v.as_str())
        .collect::<Vec<_>>();
    let locations = matches.get_many::<String>("REMOTE_LOCATION")
        .unwrap_or_default()
        .map(|v| v.to_owned())
        .collect::<Vec<_>>();

    for server in servers {
        Runtime::new().unwrap()
            .block_on(client.get_server(server).delete_files(locations.to_owned()))
            .expect("Error while deleting remote file location");
    }

}

fn handle_upload(client: &Client, matches: &ArgMatches) {
    let remote = matches.get_one::<RemotePath>("REMOTE_LOCATION").unwrap();
    let local = matches.get_one::<String>("LOCAL_LOCATION").unwrap();

    let upload_url = Runtime::new().unwrap().block_on(client.get_server(&remote.uid).get_files_upload_url());
    match upload_url {
        Ok(url) => {
            let form = http::multipart::Form::new().file("files", local);

            let mut request = http::Client::new()
                .post(url)
                .multipart(form.unwrap());

            if &remote.path != "/" || &remote.path != "." {
                request = request.query(&[("directory", &remote.path)])
            }

            let response = request.send().unwrap().error_for_status();

            match response {
                Ok(_) => (),
                Err(e) => panic!("Error occurred when uploading file to remote: {}", e)
            }
        },
        Err(e) => panic!("Error occurred when attempting to get signed upload URL for {}: {}", remote, e)
    }
}

fn handle_download(client: &Client, matches: &ArgMatches) {
    let remote = matches.get_one::<RemotePath>("REMOTE_LOCATION").unwrap();
    let local = matches.get_one::<String>("LOCAL_LOCATION").unwrap();

    let download_url = Runtime::new().unwrap().block_on(client.get_server(&remote.uid).get_file_download_url(&remote.path));
    match download_url {
        Ok(url) => {
            let response = http::get(url).unwrap().error_for_status();
            match response {
                Ok(mut url_response) => {
                    let mut file = File::create(local).unwrap();
                    let remote_data = url_response.copy_to(&mut file);
                    match remote_data {
                        Ok(_) => (),
                        Err(e) => panic!("Error occurred while saving file: {}",e)
                    }
                },
                Err(e) => panic!("Error occurred when getting remote file: {}", e)
            }
        },
        Err(e) => panic!("Error occurred when attempting to get signed download URL for {}: {}", remote, e)
    }
}

fn handle_send_command(client: &Client, matches: &ArgMatches) {
    let servers = matches.get_many::<String>("servers")
        .unwrap_or_default()
        .map(|v| v.as_str())
        .collect::<Vec<_>>();
    let command = matches.get_one::<String>("command").unwrap();

    for server in servers {
        Runtime::new().unwrap()
            .block_on(client.get_server(server).send_command(command))
            .expect("Error occurred while sending console command");
    }
}

fn handle_power_signal(client: &Client, matches: &ArgMatches) {
    let servers = matches.get_many::<String>("servers")
        .unwrap_or_default()
        .map(|v| v.as_str())
        .collect::<Vec<_>>();
    let power_signal = matches.get_one::<String>("POWER_SIGNAL").unwrap();

    match power_signal.as_ref() {
        "kill" => {
            for server in servers {
                send_power_signal(client, server, PowerSignal::Kill);
            }
        },
        "start" => {
            for server in servers {
                send_power_signal(client, server, PowerSignal::Start);
            }
        },
        "stop" => {
            for server in servers {
                send_power_signal(client, server, PowerSignal::Stop);
            }
        },
        "restart" => {
            for server in servers {
                send_power_signal(client, server, PowerSignal::Restart);
            }
        },
        _ => {
            // This case should never match when using clap cli
            panic!("Unrecognized power signal argument. This should not happen, please report")
        }
    }
}

fn send_power_signal(client: &Client, server: &str, signal: PowerSignal) {
    Runtime::new().unwrap()
        .block_on(client.get_server(server).send_power_signal(signal))
        .expect("Error occurred when sending powersignal");
}