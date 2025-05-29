
# VITIS
 
A web service for managing all project related tasks for acelrtech.

## Tech Stack
| Type | Technologies |
|---|---|
| Server | Rust (Actix-web), Bash |
| Database | PostgreSQL |
| API Documention | OpenAPI Swagger |


## CUSTOM COMMAND FOR DEBUG:
### FOR MIGRATION:
```
cargo run --bin vitis -- migrate
```

### FOR TOKEN GENERATION:
```
cargo run --bin vitis -- generate_token
```

## CUSTOM COMMAND FOR RELEASE:
### FOR MIGRATION:

    cargo run --release --bin  vitis -- migrate

    OR 

    ./target/release/vitis migrate

### FOR TOKEN GENERATION:
```
cargo run --release --bin  rapid -- generate_token

OR 

./target/release/vitis generate_token
```

## SQLX OFFLINE MODE:

```
cargo sqlx prepare
```

## ENVIRON VARIABLE 
- Set the following environ variables in `env.sh`
- `env.sh`:
```

## DATABASE VARIABLES
export DATABASE__PASSWORD=""
export DATABASE__PORT=5000
export DATABASE__HOST=""
export DATABASE__NAME=""
export DATABASE__TEST_NAME=""
export DATABASE_URL="postgres://postgres:{{password}}@{{ip}}:{{port}}/{{database_name}}"
export DATABASE__USERNAME="postgres"
export DATABASE__ACQUIRE_TIMEOUT=5
export DATABASE__MAX_CONNECTIONS=2000
export DATABASE__MIN_CONNECTIONS=10

## TRACING VARIABLES
export OTEL_SERVICE_NAME="preprod-vitis"
export OTEL_EXPORTER_OTLP_TRACES_ENDPOINT="http://localhost:4317"


## SECRET VARIABLE
export SECRET__JWT__SECRET=""
export SECRET__JWT__EXPIRY=876600
export SECRET__OTP__EXPIRY=30

## APPLICATION VARIABLE
export APPLICATION__NAME=""
export APPLICATION__ACCOUNT_NAME=""
export APPLICATION__PORT=8001
export APPLICATION__HOST=0.0.0.0
export APPLICATION__WORKERS=16
export APPLICATION__SERVICE_ID="9a3c0909-3c5d-4a84-8fb6-71928e28cb5b"
## WEBSOCKET SERVICE
export WEBSOCKET__TOKEN=""
export WEBSOCKET__BASE_URL="http://0.0.0.0:8229"
export WEBSOCKET__TIMEOUT_MILLISECONDS=600000

```


- In order to verify SQL queries at compile time, set the below config in `.env` file:
```
export DATABASE_URL="postgres://postgres:{password}@{host}:{port}/{db_name}"

```

## TO RUN THE SERVER:
- For running development server:
```
bash dev_run.sh
```
- For running production server:
```
bash release.sh
```
- For killing server:
```
bash kill.sh
```

- For restarting server:
```
bash restart.sh
```


## API DOCUMENTATION:
The API Docmentation can be found at `https://{{domain}}/docs/` after running the server.


## DEBUG SETUP:
- launch.json
```json
{

    "version": "0.2.0",
    "configurations": [

        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug executable 'vitis'",
            "cargo": {
                "args": [
                    "build",
                    "--bin=vitis",
                    "--package=vitis"
                ],
                "filter": {
                    "name": "vitis",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}",
            "envFile": "${workspaceFolder}/.env",
            "preLaunchTask": "cargo build",
        },
    ]
}
```
- settings.json

```json
{
    "[rust]": {
        "editor.formatOnSave": true,
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    },
    "editor.formatOnSave": true,
    "rust-analyzer.linkedProjects": [
        "./Cargo.toml"
    ],
}
```

- tasks.json
```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "cargo build",
            "type": "shell",
            "command": "cargo",
            "args": [
                "build",
                "--bin=vitis",
                "--package=vitis"
            ],
            "group": {
                "kind": "build",
                "isDefault": true
            },
            "problemMatcher": [
                "$rustc"
            ]
        }
    ]
}
```

## MILESTONE 1:
* [x] Setup the project structure.
* [x] Create user account creation API.
* [x] Create user account Authentication API.
* [x] Create user account fetch API.
* [x] Create project creation API.
* [x] List all project of a user data API.
* [x] Fetch project data API.
* [x] Create user account deletion API.
* [x] Create user account reactivation API.
* [ ] Create user account edit API.
* [ ] Create project deletion API.
* [ ] Create project edit API.
* [ ] Fetch All Minimal User Account API.

## MILESTONE 2:
* [x] Create Setting creation API.
* [x] Create Setting fetch API.
* [ ] Create Setting edit API.


## MILESTONE 3:
* [ ] Create project task creation API.
* [ ] Create project task deletion API.
* [ ] Create project task edit API.
* [ ] Create project task assignment API.
* [ ] Create project task unassignment API.
* [ ] Create project task status update API.
* [ ] Create On-call creation API.
* [ ] Create On-call fetch API.
* [ ] Create On-call history fetch API.

## MILESTONE 4:
* [ ] Create Permission fetch API.
* [ ] Create Role fetch API.
* [ ] Create user-project association API.
* [ ] Create user-project deletion API.
* [ ] Create role permissions assigment API.
* [ ] Create role permissions assigment edit API.
* [ ] Create project fetch API.
* [ ] Create project task fetch API.