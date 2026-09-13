# Recorder

The {{ product_name }} Recorder is capable of streaming into different sinks. A sink can
be a `WebM file`, `Display`, or `RTMP stream`. The `Display` can be toggled for debug purpose in the settings. The `WebM file` and `RTMP stream` will be configured from the signaling to the `Controller`.

## Configuration

The section in the [configuration file](README.md) is called `recorder`.

| Field                   | Type                   | Required | Default value | Description                                                      |
| ----------------------- | ---------------------- | -------- | ------------- | ---------------------------------------------------------------- |
| `clock_format`          | `string`               | no       | "%x %X %Z"    | The time format for the clock, see `man strftime` for details    |
| `display`               | `bool`                 | no       | false         | Shows the current recording in an extra window (debug purpose).  |
| `hardware_acceleration` | `HardwareAcceleration` | no       | `<empty>`     | Enabled Hardware Acceleration, which enables GPU Encoding        |
| `max_load`              | `int`                  | no       | 80            | The usage value per core (in %) until when new jobs are accepted |

### Hardware Acceleration Configuration

| Field          | Type           | Required | Default value | Description                                                    |
| -------------- | -------------- | -------- | ------------- | -------------------------------------------------------------- |
| `manufacturer` | `enum (intel)` | no       | -             | Enabled Hardware Acceleration, if Manufacturer is set          |
| `device`       | `string`       | no       | -             | The device where the metrics should be fetch (only intel GPUs) |

Right now only `Intel` is support for Hardware Acceleration, if `Intel` is not set, the CPU will be used for Video Encoding.

Note: Please keep in mind to passthrough the device and capabilities, if you're using docker.

```yaml
service:
  recorder:
    image: recorder
    # ...
    cap_add:
      - CAP_PERFMON
    devices:
      - /dev/dri
```

Of if you only want to passthrough one device (in case of load balancing):

```yaml
service:
  recorder:
    image: recorder
    # ...
    cap_add:
      - CAP_PERFMON
    devices:
      - /dev/dri/renderD129
```

Note for passing through Intel Cards: Please keep in mind, to set the `device`
if you have multiple GPUs. It doesn't matter if you only passthrough one GPU
to a container, if you have multiple GPUs intel will monitor the main device.
This means that if you have multiple graphics cards, it's mandatory to set the
`device` entry.

### Examples

#### Example with Hardware Acceleration

```toml
[recorder.hardware_acceleration]
manufacturer = "intel"
device = "/dev/dri/renderD129"
```

#### Example with clock_format

Set the time format for the clock in the recording.

```toml
[recorder]
clock_format = "%d.%m.%Y %H:%M:%S"
```

#### Example with display sink

The Display sink can be used to stream from the recorder to a display.

```toml
[recorder]
display = true
```
