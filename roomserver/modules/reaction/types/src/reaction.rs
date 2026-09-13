// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reaction {
    /// 👍
    ThumbsUp,
    /// 👎
    ThumbsDown,
    /// ❤️
    Heart,
    /// 😂
    Joy,
    /// 🥲
    SmilingFaceWithTear,
    /// 😮
    OpenMouth,
    /// 🎉
    Tada,
    /// 👏
    Clap,
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::Reaction;

    #[test]
    fn serialize_thumbs_up() {
        let reaction = Reaction::ThumbsUp;
        let raw = serde_json::to_string_pretty(&reaction).unwrap();
        assert_snapshot!(raw, @r#""thumbs_up""#);
    }

    #[test]
    fn deserialize_thumbs_up() {
        let json = json!("thumbs_up");
        let reaction: Reaction = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(reaction, Reaction::ThumbsUp);
    }

    #[test]
    fn serialize_thumbs_down() {
        let reaction = Reaction::ThumbsDown;
        let raw = serde_json::to_string_pretty(&reaction).unwrap();
        assert_snapshot!(raw, @r#""thumbs_down""#);
    }

    #[test]
    fn deserialize_thumbs_down() {
        let json = json!("thumbs_down");
        let reaction: Reaction = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(reaction, Reaction::ThumbsDown);
    }

    #[test]
    fn serialize_heart() {
        let reaction = Reaction::Heart;
        let raw = serde_json::to_string_pretty(&reaction).unwrap();
        assert_snapshot!(raw, @r#""heart""#);
    }

    #[test]
    fn deserialize_heart() {
        let json = json!("heart");
        let reaction: Reaction = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(reaction, Reaction::Heart);
    }

    #[test]
    fn serialize_joy() {
        let reaction = Reaction::Joy;
        let raw = serde_json::to_string_pretty(&reaction).unwrap();
        assert_snapshot!(raw, @r#""joy""#);
    }

    #[test]
    fn deserialize_joy() {
        let json = json!("joy");
        let reaction: Reaction = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(reaction, Reaction::Joy);
    }

    #[test]
    fn serialize_smiling_face_with_tear() {
        let reaction = Reaction::SmilingFaceWithTear;
        let raw = serde_json::to_string_pretty(&reaction).unwrap();
        assert_snapshot!(raw, @r#""smiling_face_with_tear""#);
    }

    #[test]
    fn deserialize_smiling_face_with_tear() {
        let json = json!("smiling_face_with_tear");
        let reaction: Reaction = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(reaction, Reaction::SmilingFaceWithTear);
    }

    #[test]
    fn serialize_open_mouth() {
        let reaction = Reaction::OpenMouth;
        let raw = serde_json::to_string_pretty(&reaction).unwrap();
        assert_snapshot!(raw, @r#""open_mouth""#);
    }

    #[test]
    fn deserialize_open_mouth() {
        let json = json!("open_mouth");
        let reaction: Reaction = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(reaction, Reaction::OpenMouth);
    }

    #[test]
    fn serialize_tada() {
        let reaction = Reaction::Tada;
        let raw = serde_json::to_string_pretty(&reaction).unwrap();
        assert_snapshot!(raw, @r#""tada""#);
    }

    #[test]
    fn deserialize_tada() {
        let json = json!("tada");
        let reaction: Reaction = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(reaction, Reaction::Tada);
    }

    #[test]
    fn serialize_clap() {
        let reaction = Reaction::Clap;
        let raw = serde_json::to_string_pretty(&reaction).unwrap();
        assert_snapshot!(raw, @r#""clap""#);
    }

    #[test]
    fn deserialize_clap() {
        let json = json!("clap");
        let reaction: Reaction = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(reaction, Reaction::Clap);
    }
}
