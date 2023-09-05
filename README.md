# sftpandftp_importer

Utility for importing files and folders from remote servers via SFTP or FTP.

## Designed with Pterodactyl in mind
This utility was designed to be used with my custom [Pterodactyl](https://github.com/pterodactyl) Panel modification, although it can be used as a standalone utility.

## Features

- Supports SFTP and FTP.
- Prompts before overwriting files.
- Small binary size.

## Usage
```
./sftpandftp_importer --help
SFTP and FTP Importer

Usage: sftpandftp_importer [OPTIONS] --user <USER> --pass <PASS> --remoteServer <REMOTE_SERVER>

Options:
      --protocol <PROTOCOL>           Protocol to use: ftp or sftp [default: ftp]
      --user <USER>                   
      --pass <PASS>                   
      --remoteServer <REMOTE_SERVER>  
      --port <PORT>                   [default: 21]
      --remoteFolder <REMOTE_FOLDER>  Remote folder to download from [default: /]
      --targetFolder <TARGET_FOLDER>  Local folder to download to [default: ./]
  -h, --help                          Print help
  -V, --version                       Print version
```

### Usage Example
```
./sftpandftp_importer --user <USER> --pass <PASS> --remoteServer <REMOTE_SERVER> --port <PORT> --remoteFolder <REMOTE_FOLDER> --targetFolder <TARGET_FOLDER>
```

### SFTP Example
```
./sftpandftp_importer --protocol=sftp --user=bf6fe251 --pass=coems --port=22 --remoteServer=ams1.furweb.com --remoteFolder=/ --targetFolder=./testingsftp
```

## Compile
```
cargo build --release
```

### Optimize (Optional)
Reduce the compiled binary size.
```
cargo build --release

# Install upx if you haven't already
sudo apt-get update
sudo apt-get install upx

upx --best sftpandftp_importer
```

## Notes 
This is how it can be used with my custom Pterodactyl Panel modification.

`startup.sh`:
```
#!/bin/sh
curl -L https://github.com/bunfloof/sftpandftp_importer/releases/download/Release/main -o sftpandftp_importer
chmod +x sftpandftp_importer
./sftpandftp_importer --protocol=sftp --user=bf6fe251 --pass=coems --port=22 --remoteServer=ams1.furweb.com --remoteFolder=/ --targetFolder=./testingsftp
rm sftpandftp_importer
rm -- "$0"
```
