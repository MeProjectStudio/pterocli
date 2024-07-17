use std::fmt;
use std::str::FromStr;

use clap::{arg, ArgAction, command, Command, value_parser};
use clap::builder::PossibleValue;

mod client;

#[derive(Clone)]
struct RemotePath {
    uid: String,
    path: String
}

impl FromStr for RemotePath {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();

        if parts.len() != 2 {
            return Err("Invalid format. Expected \"uid:remote/path/to/file\"");
        }

        Ok(RemotePath {
            uid: parts[0].to_string(),
            path: parts[1].to_string(),
        })
    }
}

impl fmt::Display for RemotePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.uid, self.path)
    }
}


fn main() {
    let matches = command!()
        .about("Command line interface for Pterodactyl API")
        .propagate_version(true)
        .subcommand(
            Command::new("client")
                .about("Client API")
                .visible_alias("cl")
                .propagate_version(true)
                .arg(
                    arg!(-'p' --"panel-url" <PANEL_URL> "URL pointing to Pterodactyl instance")
                        .required(true)
                        .env("PTEROCLI_PANEL_URL")
                )
                .arg(
                    arg!(-'c' --"client-key" <CLIENT_KEY> "Pterodactyl Client API token")
                        .required(true)
                        .env("PTEROCLI_CLIENT_KEY")
                )
                .arg(
                    arg!(-'s' --"servers" <SERVERS> "List of servers to perform command on. Short UIDs")
                        .value_delimiter(',')
                        .global(true)
                )
                .subcommand(
                    Command::new("powersignal")
                        .about("Sends Power Signal to specified server")
                        .arg(
                            arg!(<POWER_SIGNAL> "Power Signal to send to specified servers")
                                .required(true)
                                .value_parser([
                                    PossibleValue::new("kill"),
                                    PossibleValue::new("start"),
                                    PossibleValue::new("stop"),
                                    PossibleValue::new("restart")
                                ])
                        )
                )
                .subcommand(
                    Command::new("sendcommand")
                        .about("Sends console command to a server")
                        .arg(arg!(command: <COMMAND> "Console command to send to a server"))
                )
                .subcommand(
                    Command::new("upload")
                        .about("Uploads local file to remote location on a server")
                        .arg(
                            arg!(<LOCAL_LOCATION> "Local file to download to")
                                .conflicts_with("servers")
                                .required(true)
                        )
                        .arg(
                            arg!(<REMOTE_LOCATION> "Remote file location. Needs to be in format uid:remote/file/location")
                                .value_parser(value_parser!(RemotePath))
                                .conflicts_with("servers")
                                .required(true)
                        )

                )
                .subcommand(
                    Command::new("download")
                        .about("Downloads remote file to local location")
                        .arg(
                            arg!(<REMOTE_LOCATION> "Remote file location. Needs to be in format uid:remote/file/location")
                                .value_parser(value_parser!(RemotePath))
                                .conflicts_with("servers")
                                .required(true)
                        )
                        .arg(
                            arg!(<LOCAL_LOCATION> "Local file to download to")
                                .conflicts_with("servers")
                                .required(true)
                        )

                )
                .subcommand(
                    Command::new("rm")
                        .about("Deletes remote locations like files and folders")
                        .arg(
                            arg!(<REMOTE_LOCATION> "A list of remote locations to delete path/to/file,path/to/another/file")
                                .value_delimiter(',')
                                .required(true)
                        )

                )
                .subcommand(
                    Command::new("backup")
                        .about("Manage backups")
                        .visible_alias("bk")
                        .subcommand(
                            Command::new("create")
                                .about("Creates backup on a remote server")
                                .arg(arg!(-'n' --"name" <NAME> "Name for a backup"))
                                .arg(arg!(-'l' --"lock" "Lock created backup").action(ArgAction::SetTrue))
                        )
                        .subcommand(
                            Command::new("ls")
                                .about("List backups")
                        )
                        .subcommand(
                            Command::new("rm")
                                .about("Remove backup")
                                .arg(arg!(<BACKUP_UUID> "Backup UUID"))
                        )
                        .subcommand(
                            Command::new("get")
                                .about("Get one-time download link for a backup")
                                .arg(arg!(<BACKUP_UUID> "Backup UUID"))
                        )
                )
        ).get_matches();

    match matches.subcommand() {
        Some(("client", sub_m)) => {
            client::handle_client(sub_m)
        }
        _ => {
            panic!("Not a valid command returned from parser. This should not happen, please report")
        }
    }
}

