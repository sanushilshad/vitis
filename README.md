
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

## EMAIL VARIABLES
export EMAIL__USERNAME=""
export EMAIL__PASSWORD=""
export EMAIL__BASE_URL=""
export EMAIL__SENDER_EMAIL=""
export EMAIL__TIMEOUT_MILLISECONDS=10000
export EMAIL__PERSONAL__BASE_URL="smtp.gmail.com"
export EMAIL__PERSONAL__MESSAGE_ID_SUFFIX="mail.gmail.com"

```

## PULSAR VARIABLE
export PULSAR__TOPIC="sanu"
export PULSAR__CONSUMER="test_consumer"
export PULSAR__SUBSCRIPTION="test_subscription"
export PULSAR__URL="pulsar://localhost:6651"


## SLACK VARIABLE
export SLACK__BASE_URL="https://hooks.slack.com/services"
export SLACK__CHANNEL__LEAVE=""
export SLACK__TIMEOUT_MILLISECONDS=600000



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

## MILESTONE 1 (18/24) (USER ACCOUNT  + LEAVE MANAGEMENT + USER SETTING):
* [x] Setup the application structure.
* [x] Create user account creation API.
* [x] Create user account Authentication API.
* [x] Create user account fetch API.
* [x] Create user account deletion API.
* [x] Create user account reactivation API.
* [x] Create User Setting creation API.
* [x] Create User Setting fetch API.
* [x] Add leave request management API.  
* [x] Add causual leave approval API. 
* [x] Add leave deletion API.
* [x] Add leave fetch API.
* [x] Fetch All Minimal User Account API.
* [x] Limit No of OTP Authentication.
* [x] Create user account edit API.
* [x] Password Reset Request API.
* [x] Email Verification API.
* [x] Add Rate Limiter.
* [ ] Add empty setting to setting fetch for user and project. 
* [ ] Create User Setting edit API (Change create API to allow update).
* [ ] Create Global Setting API.
* [ ] Fetch Setting Enum API.
* [ ] Add auto-slack alert notification. 
* [ ] Add websocket notification to all APIs.


## MILESTONE 2 (4/5):
* [x] Integrate Websocket.
* [x] Integrate Email.
* [x] Integrate Pulsar.
* [x] Integrate Slack.
* [ ] Integrate Whatsapp.

## MILESTONE 3 (7/12) (PROJECT MANAGEMENT):
* [x] Create project creation API.
* [x] List all project of a user data API.
* [x] Fetch project data API.
* [x] Create Project Setting fetch API.
* [x] Create Project Setting creation API.
* [x] Create Project Setting fetch API.
* [x] Create user-project association API.
* [ ] Fetch allowed Setting detail API for a project.
* [ ] Create project deletion API (if admin else remove the association).
* [ ] Create fetch all user associated to a project API.
* [ ] Create project edit API.
* [ ] Create Project Setting edit API.


## MILESTONE 4 (0/4) (ROLE AND PERMISSION MANAGEMENT):
* [ ] Create Role fetch API.
* [ ] Create role permissions assigment API.
* [ ] Create Permission fetch API.
* [ ] Create role permissions assigment edit API.


## MILESTONE 5  (0/3) (ON-CALL MANAGEMENT):
* [ ] Create On-call creation API.
* [ ] Create On-call fetch API.
* [ ] Create On-call history fetch API.


## MILESTONE 6 (0/7) (TASK MANAGEMENT):
* [ ] Create project task assignment API.
* [ ] Create project task unassignment API.
* [ ] Create project task status update API.
* [ ] Create project task fetch API.
* [ ] Create project task creation API.
* [ ] Create project task deletion API.
* [ ] Create project task edit API.


## MILESTONE 7 (QA LIVE BUILD MILESTONE):
* [ ] In-progress


## MILESTONE 8 (Expense Management):
* [ ] In-progress





