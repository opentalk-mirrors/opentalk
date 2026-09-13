# Internal

## Configuration

The section in the [configuration file](configuration.md) is called `internal`.

| Field                             | Type            | Required | Default value | Description                                                                              |
| --------------------------------- | --------------- | -------- | ------------- | -----------------------------------------------------------------------------------------|
| `parallel_storage_quota_requests` | Non zero `uint` | no       | `5`           | The number of rooms to modify in parallel when receiving a storage quota `POST` request. |

### Parallel Storage Quota Requests

Higher numbers will reduce the time it takes to update the storage quota for all rooms,
but will increase server load while the requests are running.
