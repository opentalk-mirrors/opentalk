<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# Tools

This document describes additional tools for working with OpenTalk.

## Keycloak Theme Generator

The `keycloak-theme-generator` is a command-line tool to generate custom Keycloak themes from templates. It allows you to create branded login pages and account management interfaces for your OpenTalk deployment.

### Features

- Generate custom Keycloak themes with your branding
- Support for light and dark mode logos
- Customizable colors, fonts, and background images
- Configurable links for terms, privacy policy, and imprint
- Optional JAR packaging for direct Keycloak deployment

### Installation

The tool binaries are installed to `/opt/opentalk-compose/bin/` and symlinked to `/usr/bin/` for easy access:

```bash
/usr/bin/keycloak-theme-generator -> /opt/opentalk-compose/bin/keycloak-theme-generator
```

### Usage

```bash
keycloak-theme-generator [OPTIONS] --name <NAME> --logo <LOGO> --logo-dark <LOGO_DARK> --fav-icon <FAV_ICON> --background-image <BACKGROUND_IMAGE>
```

### Required Options

| Option | Description |
|--------|-------------|
| `-n, --name <NAME>` | Theme name (e.g., "mycompany") |
| `-l, --logo <LOGO>` | Path to logo image (light mode) |
| `-d, --logo-dark <LOGO_DARK>` | Path to logo image (dark mode) |
| `-f, --fav-icon <FAV_ICON>` | Path to favicon |
| `-b, --background-image <BACKGROUND_IMAGE>` | Path to background image |

### Optional Options

| Option | Description |
|--------|-------------|
| `-D, --display-name <DISPLAY_NAME>` | Display name for the theme |
| `-p, --primary-color <PRIMARY_COLOR>` | Primary color (hex format, e.g., "#d1e545") |
| `-s, --secondary-color <SECONDARY_COLOR>` | Secondary color (hex format, e.g., "#20434f") |
| `-F, --font-family <FONT_FAMILY>` | Font family name |
| `--terms-url <TERMS_URL>` | Terms and conditions URL |
| `--privacy-url <PRIVACY_URL>` | Privacy policy URL |
| `--imprint-url <IMPRINT_URL>` | Imprint/legal notice URL |
| `-o, --output <OUTPUT>` | Output directory (defaults to `./output/<theme-name>`) |
| `-j, --jar` | Package theme as JAR file for Keycloak deployment |

### Examples

#### Basic Theme Generation

Generate a theme with minimal required options:

```bash
keycloak-theme-generator \
  --name mycompany \
  --logo /path/to/logo.png \
  --logo-dark /path/to/logo-dark.png \
  --fav-icon /path/to/favicon.ico \
  --background-image /path/to/background.jpg
```

#### Full Customization with JAR Output

Generate a fully customized theme packaged as a JAR file:

```bash
keycloak-theme-generator \
  --name mycompany \
  --display-name "My Company" \
  --logo /path/to/logo.png \
  --logo-dark /path/to/logo-dark.png \
  --fav-icon /path/to/favicon.ico \
  --background-image /path/to/background.jpg \
  --primary-color "#d1e545" \
  --secondary-color "#20434f" \
  --font-family "Inter" \
  --terms-url "https://example.com/terms" \
  --privacy-url "https://example.com/privacy" \
  --imprint-url "https://example.com/imprint" \
  --output ./my-theme \
  --jar
```

### Deploying the Theme

After generating the theme:

1. **Without JAR**: Copy the generated theme directory to your Keycloak themes folder (`/opt/keycloak/themes/`)
2. **With JAR**: Deploy the generated JAR file to Keycloak's providers directory (`/opt/keycloak/providers/`)
3. **Rebuild Keycloak**: Run `kc.sh build` to import the new theme
4. **Restart Keycloak**: Restart the Keycloak service to apply changes

Then configure the theme in Keycloak:

1. Navigate to **Realm Settings** > **Themes**
2. Select your custom theme from the dropdown for Login, Account, or other theme types
3. Save the changes
