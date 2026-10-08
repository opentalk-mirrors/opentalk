# Monitoring

The {{ product_name }} Orchestrator can serve health probe endpoints on a separate port, e.g. for Kubernetes liveness
and readiness probes.

Endpoints provided:

| Endpoint       | Purpose         | Response                                              |
| -------------- | --------------- | ----------------------------------------------------- |
| `/`, `/health` | Liveness probe  | `200` while the orchestrator is running               |
| `/ready`       | Readiness probe | `200` when the orchestrator is ready, otherwise `503` |

## Configuration

The section in the [configuration file](README.md) is called `monitoring`. The section is optional. Monitoring is
disabled when it is not present.

| Field     | Type     | Required | Default value | Description                                            |
| --------- | -------- | -------- | ------------- | ------------------------------------------------------ |
| `address` | `string` | no       | `0.0.0.0`     | IP address to which to listen for incoming connections |
| `port`    | `uint`   | no       | `11221`       | TCP port number where the probe endpoints are served   |

### Example

```toml
[monitoring]
address = "0.0.0.0"
port = 11221
```
