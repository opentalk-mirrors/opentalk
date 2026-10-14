# File Storage (MinIO)

The {{ product_name }} Controller uses [MinIO](https://min.io/) as its object storage.

## Configuration

The section in the [configuration file](./configuration.md) is called `minio`.

| Field              | Type     | Required | Default value | Description                                                                                                                                                      |
| ------------------ | -------- | -------- | ------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `uri`              | `string` | yes      | -             | The URI address where the MinIO server can be reached                                                                                                            |
| `bucket`           | `string` | yes      | -             | The default bucket name for object storage                                                                                                                       |
| `access_key`       | `string` | yes      | -             | The unique username for the MinIO user                                                                                                                           |
| `secret_key`       | `string` | yes      | -             | The password corresponding to the access key                                                                                                                     |
| `force_path_style` | `bool`   | no       | false         | Use path-style bucket URLs (`host/bucket`) instead of virtual-hosted style; set to `true` for S3-compatible servers that don't support virtual-hosted addressing |
| `region`           | `string` | no       | unknown       | AWS region                                                                                                                                                       |

### Examples

#### Default setup

```toml
[minio]
uri = "http://localhost:9555"
bucket = "controller"
access_key = "minioadmin"
secret_key = "minioadmin"
force_path_style = false
region = "unknown"
```
