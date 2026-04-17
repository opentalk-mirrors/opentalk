<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# Managing OpenTalk Users in Keycloak

## Method A: Creating Users via the Keycloak Admin Console (GUI)

The Keycloak Admin Console provides a user-friendly graphical interface for managing users. Follow these steps to create a new user:

1. **Access the Keycloak Admin Console**  
    Open your web browser and navigate to the Keycloak Admin Console. For detailed instructions, refer to [Section 3.1: Keycloak Admin Console](./2-access-web-interfaces.md#21-keycloak-admin-console).

2. **Log In with Administrator Credentials**  
    Enter your administrator username and password to log in.

3. **Select the Appropriate Realm**  
    In the left sidebar, locate the realm selector and choose the realm where you want to create the user. By default, this is usually named `opentalk`.

4. **Navigate to User Management**  
    Click on "Users" in the main menu. Then click the "Add user" button to begin creating a new user account.

5. **Enter User Information**  
    Fill in the required fields:  
    - **Username:** The unique identifier for the user.  
    - **Email:** The user's email address.  
    - **First Name:** The user's first name.  
    - **Last Name:** The user's last name.  
    Ensure the "Enabled" option is selected to activate the user immediately. Click "Create" to proceed.

6. **Set the User's Password**  
    After creating the user, switch to the "Credentials" tab. Enter a password for the user. If you want the password to be permanent, disable the "Temporary" option. Otherwise, the user will be prompted to change the password upon first login.

7. **Optional: Additional Actions**  
    - Send a verification email by clicking "Send verify email".  
    - Trigger a password reset if required.

Once these steps are completed, the user account is active. The user can now log in to the OpenTalk Dashboard using their credentials.

---

## Method B: Creating Users via CLI in the Keycloak Container (via kcadm.sh)

For advanced use or automation, Keycloak provides a command-line tool (`kcadm.sh`) that can be used within the Keycloak container. The following steps outline how to create and manage users using the CLI:

1. **Authenticate in the Keycloak Container**  
    Access the Keycloak container and authenticate with admin credentials.  
    The following example authenticates against the `master` realm. This is appropriate because the `admin` user belongs to the `master` realm and has permission to manage other realms:

    ```bash
    docker compose exec keycloak /opt/keycloak/bin/kcadm.sh config credentials \
      --server http://localhost:8080 --realm master \
      --user admin --password "$KC_PERMANENT_ADMIN_PASSWORD"
    ```

2. **Create a New User**  
    Adjust the realm as needed (default: `opentalk`). Run:

    ```bash
    docker compose exec keycloak /opt/keycloak/bin/kcadm.sh create users -r opentalk \
      -s username=j.doe -s email=j.doe@example.org -s firstName=Jane -s lastName=Doe -s enabled=true
    ```

    Assign a password to the newly created user:

    ```bash
    docker compose exec keycloak /opt/keycloak/bin/kcadm.sh set-password -r opentalk \
      --username j.doe --new-password 'S3curePW!' --temporary false
    ```

---

## Method C: Bulk User Import via otctl

For importing a larger number of users at once, `otctl` provides the `import users` command. Users can be defined in a CSV or JSON file and are created in Keycloak automatically.

### File Formats

**CSV format** — the first row must be a header with the following columns:

```csv
username,email,firstName,lastName,password,enabled,tariff_id,tariff_status,tenant_id,password_reset_email
j.doe,j.doe@example.org,Jane,Doe,S3curePW!,true,OpenTalkDefaultTariff,,,false
m.mustermann,m.mustermann@example.org,Max,Mustermann,An0therPW!,true,OpenTalkDefaultTariff,,default,false
a.schmidt,a.schmidt@example.org,Anna,Schmidt,,true,,,,true
```

**JSON format** — the file must contain a JSON array of user objects:

```json
[
  {
    "username": "j.doe",
    "email": "j.doe@example.org",
    "firstName": "Jane",
    "lastName": "Doe",
    "password": "S3curePW!",
    "enabled": true,
    "tariff_id": "OpenTalkDefaultTariff",
    "tenant_id": "",
    "password_reset_email": false
  }
]
```

**Required fields:** `username`, `email`, `firstName`, `lastName`

**Optional fields:**

| Field | Description | Default |
| ------- | ------------- | --------- |
| `password` | User password. If empty, `--default-password` is used; if that is also unset, a random password is auto-generated (see below). Ignored when `password_reset_email` is `true`. | *(auto-generated)* |
| `enabled` | Whether the user account is enabled. | `true` |
| `tariff_id` | OpenTalk tariff identifier (e.g. `OpenTalkDefaultTariff`). | *(empty)* |
| `tariff_status` | OpenTalk tariff status. | *(empty)* |
| `tenant_id` | OpenTalk tenant identifier. | *(empty)* |
| `password_reset_email` | When `true`, no initial password is set and Keycloak sends the user an email with a link to set their own password. The `password` field is ignored for these users. | `false` |

Example files are shipped with the package at `/usr/share/opentalk-compose/examples/users.csv-sample` and `/usr/share/opentalk-compose/examples/users.json-sample`.

### Usage

```bash
# Import users from a CSV file
otctl import users --users-file=/path/to/users.csv

# Import users from a JSON file
otctl import users --users-file=/path/to/users.json

# Make passwords permanent (users will NOT be prompted to change on first login)
otctl import users --users-file=/path/to/users.csv --no-temporary

# Force all users to receive a Keycloak password-reset email, regardless of the
# per-user 'password_reset_email' field in the import file
otctl import users --users-file=/path/to/users.csv --password-reset-email

# Set a default password for all users that don't have one in the file
otctl import users --users-file=/path/to/users.csv --default-password='ChangeMe!'

# Auto-generate passwords with custom length (default: 20 characters)
otctl import users --users-file=/path/to/users.csv --password-length=32
```

By default, all imported passwords are marked as **temporary** — users must change their password on first login. Use `--no-temporary` to override this behavior.

The `--password-reset-email` flag is a **hard override**: it forces the email-based password reset for every user in the file, even if individual records have `password_reset_email: false`. Without the flag, the per-user field in the import file is used.

#### Password resolution order

When a user record has no `password` field (or it is empty), the password is determined in this order:

1. **`--password-reset-email`** — if set (globally or per-user), no password is assigned. The user receives an email to set their own password.
2. **`--default-password=X`** — the given password X is used for all users without a password.
3. **Auto-generation** — a random password of `--password-length` characters (default: 20) is generated and printed to the console.

!!! note
    The import is **additive and idempotent**: existing users with the same username are updated, no users are deleted.

---

## Additional Notes

- If SMTP is not configured, invitation and verification emails will not be sent.
- User permissions depend on assigned roles and group memberships.
- Some changes may only become visible after the user logs out and logs back in due to caching.

---

## Troubleshooting

- **401 or 403 errors with kcadm.sh:** Verify realm and admin credentials.
- **Login issues:** Ensure the user is enabled and the password is not still marked temporary.
- **Missing features:** Check that the user has the required roles and group memberships.
- **No emails sent:** Confirm SMTP settings are configured correctly in Keycloak.
