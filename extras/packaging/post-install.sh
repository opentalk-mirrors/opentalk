#!/bin/bash
mkdir -p /var/opt/opentalk-compose/data
mkdir -p /var/opt/opentalk-compose/backups
chmod 755 /var/opt/opentalk-compose/data
chmod 755 /var/opt/opentalk-compose/backups
chown root:root /var/opt/opentalk-compose/data
chown root:root /var/opt/opentalk-compose/backups