# Load Tests

This directory contains the k6-based regression and stress tests for the OpenTalk roomserver. The scripts are plain JavaScript (ESM) and rely on helpers under `src/lib/` to keep scenarios concise. Each load tests is a single JavaScript file under `src/tests/`.

## Running Tests Locally

You can run the tests locally using docker.
To do so, you have to edit your roomserver config file and set the `public_url` to `http://host.docker.internal:11333`.
Then follow the steps below:

```bash
# start the roomserver in release mode (from the root directory)
cargo run --release
# navigate into the load-test directory
cd load-test
# run a load test, in this case fairness.js
# this will also automatically start containers for prometheus and grafana
sudo docker compose run --rm k6 fairness.js
```

You can view the live results in [Grafana](http://localhost:9000/dashboards).

## Deploying Load Tests

Running the RoomServer and k6 on the same machine will not yield meaningful results as you won't be able to differentiate between the two.
You need two separate servers, one running the RoomServer (device under test) and one running k6 (test device).

## Deploying the test device

Copy the `load-test` directory to the machine. Edit the [`.env`](./.env) file and replace the value of `LOAD_TEST_BASE_URL` with the URL under which the RoomServer will be reachable (on the device under test).
Because prometheus does currently not support environment variable substitution, it is necessary to manually edit the [`prometheus.yaml`](./prometheus/prometheus.yaml) file and replace the target of the RoomServer URL with the same value you specified for `LOAD_TEST_BASE_URL`.

## Deploying the device under test
