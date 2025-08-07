# Managing OpenTalk Users in Keycloak

## Table of Contents

- [Method A: Creating Users via the Keycloak Admin Console (GUI)](#method-a-creating-users-via-the-keycloak-admin-console-gui)
- [Method B: Creating Users via CLI in the Keycloak Container (via kcadm.sh)](#method-b-creating-users-via-cli-in-the-keycloak-container-via-kcadmsh)
- [Additional Notes](#additional-notes)
- [Troubleshooting](#troubleshooting)


## Method A: Creating Users via the Keycloak Admin Console (GUI)

The Keycloak Admin Console provides a user-friendly graphical interface for managing users. Follow these steps to create a new user:

1. **Access the Keycloak Admin Console**  
	Open your web browser and navigate to the Keycloak Admin Console. For detailed instructions on accessing the console, refer to [Section 3.1: Keycloak Admin Console](./2-access-web-interfaces.md#31-keycloak-admin-console).

2. **Log In with Administrator Credentials**  
	Enter your administrator username and password to log in.

3. **Select the Appropriate Realm**  
	In the left sidebar, locate the "Manage Realm" section and select the realm where you want to create the user. By default, this is usually named `opentalk`.

4. **Navigate to User Management**  
	Click on "Users" in the main menu. Then, click the "Add user" button to begin creating a new user account.

5. **Enter User Information**  
	Fill in the required fields:
	- **Username:** The unique identifier for the user.
	- **Email Address:** The user's email address.
	- **First Name:** The user's first name.
	- **Last Name:** The user's last name.  
	Ensure the "Enabled" option is selected to activate the user immediately. Click "Create" to proceed.

6. **Set the User's Password**  
	After creating the user, switch to the "Credentials" tab. Enter a password for the user. If you want the password to be permanent, disable the "Temporary" option. Otherwise, the user will be prompted to change their password upon first login.

7. **Optional: Additional Actions**  
	- You can send a verification email to the user by clicking "Send verify email".
	- You may also trigger a password reset if needed.

Once these steps are completed, the user account is active. The user can now log in to the OpenTalk Dashboard using their credentials.

---

## Method B: Creating Users via CLI in the Keycloak Container (via kcadm.sh)

For advanced users or automation scenarios, Keycloak provides a command-line tool (`kcadm.sh`) that can be used within the Keycloak container. The following steps outline how to create and manage users using the CLI:

1. **Authenticate in the Keycloak Container**  
	First, access the Keycloak container and authenticate with your admin credentials:
	The following example demonstrates authentication to the `master` realm. This is appropriate because the `admin` user belongs to the `master` realm and has permission to manage other realms.

	```bash
	docker compose exec keycloak /opt/keycloak/bin/kcadm.sh config credentials \
	  --server http://localhost:8080 --realm master \
	  --user admin --password "$KC_BOOTSTRAP_ADMIN_PASSWORD"
	```

2. **Create a New User**  
	Adjust the realm as needed (default: `opentalk`). Run the following command to create a user:
	```bash
	docker compose exec keycloak /opt/keycloak/bin/kcadm.sh create users -r opentalk \
	  -s username=j.doe -s email=j.doe@example.org -s firstName=Jane -s lastName=Doe -s enabled=true
	```

3. **Set the User's Password**  
	Assign a password to the newly created user:
	```bash
	docker compose exec keycloak /opt/keycloak/bin/kcadm.sh set-password -r opentalk \
	  --username j.doe --new-password 'S3curePW!' --temporary false
	```

---

## Additional Notes

- If SMTP is not configured, invitation and verification emails will not be sent.
- User permissions depend on the roles and groups assigned.
- Changes may only become visible after the user logs out and logs back in, due to caching in browsers or Keycloak.

---

## Troubleshooting

- **401/403 errors in kcadm:** Check that you are using the correct realm and admin credentials.
- **Login issues:** Ensure the user is enabled and the password is not set as temporary.
- **Missing features:** Verify that the user has the necessary roles and group memberships.
- **No emails sent:** Confirm that SMTP parameters are correctly configured in Keycloak.
