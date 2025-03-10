#!/bin/bash
mkdir -p /var/opt/opentalk-compose/data
mkdir -p /var/opt/opentalk-compose/backups
chmod 755 /var/opt/opentalk-compose/data
chmod 755 /var/opt/opentalk-compose/backups
chown root:root /var/opt/opentalk-compose/data
chown root:root /var/opt/opentalk-compose/backups

ANSIBLE_DEFAULT_FILE=/opt/opentalk-compose/services/cforge/ansible/roles/opentalk_config/defaults/main.yml
SETTINGS_FILE=/opt/opentalk-compose/config/settings.yml
MAN_FILE=/usr/share/man/man1/opentalk-compose.1
CFORGE_FILE=/usr/bin/cforge
BASH_COMPLETION_CFORGE=/etc/bash_completion.d/cforge.bash

if [ ! -f $SETTINGS_FILE ]; then
awk 'NR==1 {print $0} 
     NR==1 {
         print "";
         print "# WARNING: Proceed with caution!";
         print "# This configuration file contains critical settings for the application.";
         print "# Modifying values without a proper understanding may lead to unexpected behavior,";
         print "# application malfunctions, or require services to be reinitialized.";
         print "# Only change settings if absolutely necessary and ensure you have a backup before making modifications.";
         print "";
     } 
     NR>1 {
         # If line does not start with #, comment it out.
         if ($0 !~ /^#/) {
             print "#" $0
         } else {
             print $0
         }
     }' "$ANSIBLE_DEFAULT_FILE" > "$SETTINGS_FILE"
fi

if [ ! -f "$MAN_FILE" ]; then
  ln -s /opt/opentalk-compose/extras/packaging/man/opentalk-compose.1 ${MAN_FILE}
fi

if [ ! -f "$CFORGE_FILE" ]; then
  ln -s /opt/opentalk-compose/extras/packaging/cforge ${CFORGE_FILE}
fi

if [ ! -f "$BASH_COMPLETION_CFORGE" ]; then
  ln -s /opt/opentalk-compose/extras/packaging/cforge_completion.bash ${BASH_COMPLETION_CFORGE}
fi

if [ -f "$BASH_COMPLETION_CFORGE" ]; then
    # shellcheck source=/dev/null
    source $BASH_COMPLETION_CFORGE
    complete -F _cforge_completion cforge
fi