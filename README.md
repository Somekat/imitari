# Imitari
Imitari is a project to provide an easy and lightweight server for images. It is compatible with software like [ShareX](https://github.com/ShareX/ShareX).
The projects goals are to provide fast, efficient, and an easy way to deploy own image upload server.

## Features

- Simple web ui with login and uploader
- Normal HTTP uploader
- Full [ShareX](https://github.com/ShareX/ShareX) support (see the example file)
- Automatic file compression
- Speed and Efficiency (Powered by [Actix](https://github.com/actix/actix) and Rust)

## Env File
The `.env` file should be placed
it will auto load an env in the same directory, or if there is none it will just get env variatbles. All config variatbles are very important

```env
BASE_URL="url used in json response"
AUTH_TOKEN="api token for rest api"
AUTH_USER="username for web"
AUTH_PASSWORD="password for web"
NAME="name of your service"
URL="0.0.0.0"
PORT=9000 # Any port you want this readme covers with 9000 port
```

## Note about AUTH_TOKEN

To generate your `AUTH_TOKEN` on Linux you must use this command `openssl rand -base64 32` (needs OpenSSL installed)
It will generate for use a base64 encrypted token and use it in AUTH_TOKEN

## Running in Docker

To run this software on Docker you must have installed the Docker itself and docker-compose.
The compose file was configurated to work with [Traefik](https://github.com/traefik/traefik) reverse proxy.
Open the `docker-compose.yml` file and inside of it, edit the values.
After you are done, just run

```shell
docker-compose up -d
```

And there you have a running instance in background.

## Running the Binaries

1. Choose the architecture. If you need them go here https://nightly.link/imitari/imitari/workflows/binaries/main
2. Download the file based on architecture
3. Extract the \*.zip file and go to the directory where you extracted Imitari.
4. Create the .env file for configuration
5. Run the executable and look for any logs!

## Deploy to Heroku

[![Deploy](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy?template=https://github.com/imitari/imitari)

## Contributing

Just fork the repo, make your own changes and then run with cargo and then create a pull request.

## Issue or bugs

Just simply report them here.
