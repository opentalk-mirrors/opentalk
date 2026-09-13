// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use serde::{Deserialize, Serialize};

use crate::{command::Kind, event::TimerEvent};

/// Incoming websocket messages
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum TimerCommand {
    /// Start a new timer
    Start {
        /// The timer kind
        #[serde(flatten)]
        kind: Kind,
        /// An optional string tag to flag this timer with a custom style
        style: Option<String>,
        /// An optional title for the timer
        title: Option<String>,
        /// Flag to allow/disallow participants to mark themselves as ready
        #[serde(default)]
        enable_ready_check: bool,
    },
    /// Stop a running timer
    Stop { reason: Option<String> },
    /// Update the ready status
    UpdateReadyStatus { status: bool },
}

impl CreateReplica<TimerEvent> for TimerCommand {
    fn replicate(&self) -> Option<TimerEvent> {
        None
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::command::kind::Kind;

    #[test]
    fn countdown_start() {
        let json = json!({
            "action": "start",
            "kind": "countdown",
            "duration": 5,
            "style": "coffee_break",
            "title": null,
            "enable_ready_check": false
        });

        assert_eq!(
            json,
            serde_json::to_value(TimerCommand::Start {
                kind: Kind::Countdown { duration: 5 },
                style: Some("coffee_break".into()),
                title: None,
                enable_ready_check: false
            })
            .unwrap()
        );
    }

    #[test]
    fn stopwatch_start() {
        let json = json!({
            "action": "start",
            "kind": "stopwatch",
            "title": "Testing the timer!",
            "style": null,
            "enable_ready_check": false
        });

        assert_eq!(
            json,
            serde_json::to_value(TimerCommand::Start {
                kind: Kind::Stopwatch,
                style: None,
                title: Some("Testing the timer!".into()),
                enable_ready_check: false
            })
            .unwrap()
        );
    }

    #[test]
    fn stop() {
        let json = json!({
            "action": "stop",
            "reason": "test"
        });

        assert_eq!(
            json,
            serde_json::to_value(TimerCommand::Stop {
                reason: Some("test".into())
            })
            .unwrap()
        );
    }

    #[test]
    fn update_ready_status() {
        let json = json!({
            "action": "update_ready_status",
            "status": true
        });

        assert_eq!(
            json,
            serde_json::to_value(TimerCommand::UpdateReadyStatus { status: true }).unwrap()
        )
    }
}
