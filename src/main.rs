use clap::Parser;
use std::fs::{self};
use std::io::{self, Read};
use std::net::TcpStream;
use std::path::Path;

extern crate ftp;
use ftp::FtpStream;

extern crate ssh2;
use ssh2::Session;

const MAX_RETRIES: usize = 5;

static mut OVERWRITE_ALL: bool = false;
static mut SKIP_ALL: bool = false;

#[derive(Debug, Parser)]
#[clap(
    name = "SFTPAndFTPImporter",
    author = "Bun",
    version = "0.1.0",
    about = "SFTP and FTP Importer"
)]
struct Args {
    /// Protocol to use: ftp or sftp
    #[clap(long, default_value = "ftp")]
    protocol: String,

    #[clap(long)]
    user: String,

    #[clap(long)]
    pass: String,

    #[clap(long = "remoteServer")]
    remote_server: String,

    #[clap(long, default_value = "21")]
    port: u16,

    /// Remote folder to download from
    #[clap(long = "remoteFolder", default_value = "/")]
    remote_folder: String,

    /// Local folder to download to
    #[clap(long = "targetFolder", default_value = "./")]
    target_folder: String,
}

fn main() {
    let args = Args::parse();

    println!("Starting Importer...");

    let address = format!("{}:{}", args.remote_server, args.port);
    println!("Connecting to {}...", address);

    match args.protocol.as_str() {
        "ftp" => handle_ftp(
            &address,
            &args.user,
            &args.pass,
            &args.remote_folder,
            &args.target_folder,
        ),
        "sftp" => handle_sftp(
            &address,
            &args.user,
            &args.pass,
            &args.remote_folder,
            &args.target_folder,
        ),
        _ => println!("Unsupported protocol"),
    }

    println!("The program has completed running. Type 'stop' or 'exit' to close the program.");
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim() == "stop" || input.trim() == "exit" {
            break;
        }
    }
}

fn handle_ftp(address: &str, user: &str, pass: &str, remote_folder: &str, target_folder: &str) {
    let mut ftp_stream = None;
    for _ in 0..MAX_RETRIES {
        match FtpStream::connect(address) {
            Ok(stream) => {
                ftp_stream = Some(stream);
                break;
            }
            Err(e) => {
                if e.to_string().contains("421") {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                } else {
                    println!("Error connecting to FTP: {}", e);
                    return;
                }
            }
        }
    }

    let mut ftp_stream = match ftp_stream {
        Some(stream) => stream,
        None => {
            println!("Failed to connect after {} retries", MAX_RETRIES);
            return;
        }
    };

    for _ in 0..MAX_RETRIES {
        match ftp_stream.login(user, pass) {
            Ok(_) => {
                println!("Logged in. Starting download...");
                break;
            }
            Err(e) => {
                if e.to_string().contains("421") {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                } else {
                    println!("Error logging in: {}", e);
                    return;
                }
            }
        }
    }

    download_folder_ftp(&mut ftp_stream, remote_folder, target_folder);
}

fn download_folder_ftp(ftp_stream: &mut FtpStream, remote_folder: &str, target_folder: &str) {
    if let Err(e) = ftp_stream.cwd(remote_folder) {
        println!("Error changing directory: {}", e);
        return;
    }

    let entries = match ftp_stream.list(None) {
        Ok(entries) => entries,
        Err(e) => {
            println!("Error listing directory: {}", e);
            return;
        }
    };

    for entry in &entries {
        println!("Processing entry: {}", entry);
        let entry_name = entry.split_whitespace().last().unwrap_or("");
        if entry_name == "." || entry_name == ".." {
            continue;
        }

        let local_target = Path::new(target_folder).join(entry_name);
        if entry.starts_with("d") {
            fs::create_dir_all(&local_target).unwrap();
            download_folder_ftp(
                ftp_stream,
                &Path::new(remote_folder).join(entry_name).to_str().unwrap(),
                &local_target.to_str().unwrap(),
            );
        } else {
            download_file_ftp(ftp_stream, entry_name, &local_target.to_str().unwrap());
        }
    }

    // Return to the previous directory after processing the current directory
    ftp_stream.cwd("..").unwrap();
}

fn download_file_ftp(ftp_stream: &mut FtpStream, remote_file: &str, target_file: &str) {
    if Path::new(target_file).exists() && unsafe { !OVERWRITE_ALL && !SKIP_ALL } {
        println!("Overwrite {}? [y]es, [n]o, [A]ll, [N]one: ", target_file);
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        match choice.trim() {
            "n" => {
                println!("Skipping {}", target_file);
                return;
            }
            "A" => unsafe { OVERWRITE_ALL = true },
            "N" => {
                unsafe { SKIP_ALL = true };
                println!("Skipping {}", target_file);
                return;
            }
            "y" => {}
            _ => {
                println!("Invalid choice. Skipping {}", target_file);
                return;
            }
        }
    }

    println!("Downloading file: {} to {}", remote_file, target_file);

    let mut reader = match ftp_stream.simple_retr(remote_file) {
        Ok(reader) => reader,
        Err(e) => {
            println!("Error retrieving file: {}", e);
            return;
        }
    };

    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();
    // Ensure the parent directory exists before writing the file
    if let Some(parent) = Path::new(target_file).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).unwrap();
        }
    }

    fs::write(target_file, buffer).unwrap();
}

fn handle_sftp(address: &str, user: &str, pass: &str, remote_folder: &str, target_folder: &str) {
    let tcp = TcpStream::connect(address).unwrap();
    let mut session = Session::new().unwrap();
    session.set_tcp_stream(tcp);
    session.handshake().unwrap();
    session.userauth_password(user, pass).unwrap();

    let mut sftp = session.sftp().unwrap();
    download_folder_sftp(&mut sftp, remote_folder, target_folder);
}

fn download_folder_sftp(sftp: &mut ssh2::Sftp, remote_folder: &str, target_folder: &str) {
    let entries = sftp.readdir(Path::new(remote_folder)).unwrap();

    for (path, stat) in entries {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let local_target = format!("{}/{}", target_folder, file_name);
        let remote_target = format!("{}/{}", remote_folder, file_name);

        if stat.is_dir() {
            fs::create_dir_all(&local_target).unwrap();
            download_folder_sftp(sftp, &remote_target, &local_target);
        } else {
            download_file_sftp(sftp, &remote_target, &local_target);
        }
    }
}

fn download_file_sftp(sftp: &mut ssh2::Sftp, remote_file: &str, target_file: &str) {
    if Path::new(target_file).exists() && unsafe { !OVERWRITE_ALL && !SKIP_ALL } {
        println!("Overwrite {}? [y]es, [n]o, [A]ll, [N]one: ", target_file);
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        match choice.trim() {
            "n" => {
                println!("Skipping {}", target_file);
                return;
            }
            "A" => unsafe { OVERWRITE_ALL = true },
            "N" => {
                unsafe { SKIP_ALL = true };
                println!("Skipping {}", target_file);
                return;
            }
            "y" => {}
            _ => {
                println!("Invalid choice. Skipping {}", target_file);
                return;
            }
        }
    }

    println!("Downloading file: {} to {}", remote_file, target_file);

    let mut remote_file = sftp.open(Path::new(remote_file)).unwrap();
    let mut buffer = Vec::new();
    remote_file.read_to_end(&mut buffer).unwrap();
    fs::write(target_file, buffer).unwrap();
}
