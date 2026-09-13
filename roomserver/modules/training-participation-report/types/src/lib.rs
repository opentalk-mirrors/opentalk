// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk training-participation-report module.

pub mod command;
pub mod event;
pub mod settings;
pub mod state;

pub use command::TrainingParticipationReportCommand;
pub use event::TrainingParticipationReportEvent;
use opentalk_types_common::modules::{ModuleId, module_id};
pub use opentalk_types_common::training_participation_report::TrainingParticipationReportParameterSet;
pub use state::TrainingParticipationReportState;

/// The module id for the signaling module
pub const TRAINING_PARTICIPATION_REPORT_MODULE_ID: ModuleId =
    module_id!("training_participation_report");

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{TRAINING_PARTICIPATION_REPORT_MODULE_ID, module_id};

    #[test]
    fn verify_module_id() {
        // Test that the crate name matches the module id
        // The name for this crate is a bit complex and easy to get wrong, e.g. participant vs
        // participation
        assert_eq!(
            env!("CARGO_CRATE_NAME"),
            &format!("opentalk_roomserver_types_{TRAINING_PARTICIPATION_REPORT_MODULE_ID}")
        );
        assert_eq!(
            TRAINING_PARTICIPATION_REPORT_MODULE_ID,
            module_id!("training_participation_report")
        );
    }
}
