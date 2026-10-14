# Operator information

Information regarding the operator that is responsible for user-facing
communication and legal disclosure.

These contents are presented to the user where approrpiate, e.g. in invitation
emails. The controller does not do any further processing of that information,
it will not open the data protection URL or send emails to the support email
address.

## Configuration

The section in the [configuration file](configuration.md) is called
`operator_information`.

| Field                   | Type     | Required | Default value | Description                                                        |
| ----------------------- | -------- | -------- | ------------- | ------------------------------------------------------------------ |
| `data_protection_url`   | `string` | no       | -             | The URL where users can find the data protection or privacy policy |
| `support_phone_number`  | `string` | no       | -             | The phone number users can call for support                        |
| `support_email_address` | `string` | no       | -             | The email address users can contact for support                    |

### Example configuration

```toml
[operator_information]
data_protection_url = "https://example.com/data-protection"
support_phone_number = "+493012345678"
support_email_address = "support@example.com"
```
