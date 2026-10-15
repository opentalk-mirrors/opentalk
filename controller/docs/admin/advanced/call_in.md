# Call-In

The call-in module provides a user-friendly solution that not only allows
participants to connect via telephone but also displays the necessary phone
number, ensuring a straightforward and accessible joining process.

## Configuration

The section in the [configuration file](../core/configuration.md) is called `call_in`.

| Field                   | Type     | Required | Default value | Description                                            |
| ----------------------- | -------- | -------- | ------------- | ------------------------------------------------------ |
| `tel`                   | `string` | yes      | -             | The Phone number which will be displayed to the user   |
| `mask_unmapped_numbers` | `bool`   | no       | `true`        | Mask unmapped phone numbers to protect privacy         |
| `default_country_code`  | `string` | yes      | -             | The default country code as Alpha-2 code (ISO 3166)    |

### Phone number masking

`mask_unmapped_numbers` controls whether the displayed phone number is masked so only the
first and last digits are visible (e.g., +491\*\*\*123) to protect participants' privacy.

### Examples

```toml
[call_in]
tel="03012345678"
mask_unmapped_numbers=true
default_country_code="DE"
```
