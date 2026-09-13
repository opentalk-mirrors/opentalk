// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

/// Represents the types of technical issues that can occur during the vote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechnicalIssueKind {
    /// An issue related to audio during the vote.
    Audio,

    /// An issue related to video during the vote.
    Video,

    /// An issue related to screen sharing during the vote.
    Screenshare,
}

#[cfg(test)]
mod test {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_audio_technical_issue_kind() {
        let produced = serde_json::to_value(TechnicalIssueKind::Audio).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#""audio""#);
    }

    #[test]
    fn deserialize_audio_technical_issue_kind() {
        let produced: TechnicalIssueKind = serde_json::from_value(json!("audio")).unwrap();

        let expected = TechnicalIssueKind::Audio;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_video_technical_issue_kind() {
        let produced = serde_json::to_value(TechnicalIssueKind::Video).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#""video""#);
    }

    #[test]
    fn deserialize_video_technical_issue_kind() {
        let produced: TechnicalIssueKind = serde_json::from_value(json!("video")).unwrap();

        let expected = TechnicalIssueKind::Video;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_screenshare_technical_issue_kind() {
        let produced = serde_json::to_value(TechnicalIssueKind::Screenshare).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#""screenshare""#);
    }

    #[test]
    fn deserialize_screenshare_technical_issue_kind() {
        let produced: TechnicalIssueKind = serde_json::from_value(json!("screenshare")).unwrap();

        let expected = TechnicalIssueKind::Screenshare;

        assert_eq!(produced, expected);
    }
}
