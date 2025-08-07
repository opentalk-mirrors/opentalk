_cforge_completion() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    opts="--help --diff --verbose --check"

    # Define top-level commands
    commands="init task apply pull"

    # Define tasks
    tasks="demoUser outlookClient selfSignedCerts createBackup createBackupConfig createBackupControllerDB createBackupKeycloakDB createBackupTraefik createBackupMinio restoreBackup restoreBackupConfig restoreBackupControllerDB restoreBackupKeycloakDB restoreBackupTraefik restoreBackupMinio"

    # Handle opts completion
    if [[ ${cur} == -* ]]; then
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
    fi

    # Handle command completion
    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "${commands}" -- ${cur}) )
        return 0
    fi

    # Handle task completion
    if [[ "${prev}" == "task" ]]; then
        COMPREPLY=( $(compgen -W "${tasks}" -- ${cur}) )
        return 0
    fi

    # Handle timestamp option for restore tasks
    if [[ ${COMP_CWORD} -eq 3 && ${COMP_WORDS[1]} == "task" && ${COMP_WORDS[2]} == restore* ]]; then
        COMPREPLY=( "--timestamp=" )
        return 0
    fi
}

complete -F _cforge_completion cforge