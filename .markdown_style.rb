# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# Enable all rules by default
all

# All unordered lists must use '-' consistently at all levels
rule 'MD004', :style => :dash
rule 'MD007', :indent => 4

rule 'MD029', :style => :ordered

# Disable duplicate heading check
exclude_rule 'MD024'

# Disable check that first line must be top level header as its incompatible with docusaurus
exclude_rule 'MD041'

# Disable line length limit because markdown tables can't have linebreaks in them
exclude_rule 'MD013'

# Allow code block identation to make tera templates more "readable"
exclude_rule 'MD046'

# These are triggered as false positives when used inside blockquoste
# See also https://github.com/markdownlint/markdownlint/issues/472
exclude_rule 'MD055'
exclude_rule 'MD057'
