// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{BTreeSet, HashMap, hash_map::Entry},
    time::Duration,
};

use anyhow::Context;
use opentalk_roomserver_signaling::{
    module_context::{ChannelDroppedError, ModuleContext},
    signaling_module::{
        ModuleJoinData, ModuleSwitchData, NoOp, NoSettings, PeerDataMap, SignalingModule,
        SignalingModuleDescription, SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    room_kind::RoomKind,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_roomserver_types_polls::{
    Choice, ChoiceId, Item, POLLS_MODULE_ID, PollId, Results,
    command::{PollsCommand, Vote},
    event::{Error, PollsEvent},
    state::PollsState,
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;
use tokio::sync::oneshot::Sender;

/// The maximum allowed duration of a poll
const MAX_POLL_DURATION: Duration = Duration::from_hours(24);
/// The maximum number of bytes the topic length is allowed to have
const MAX_TOPIC_LENGTH: usize = 100;
/// The minimum number of choices a poll must have
const MIN_CHOICE_COUNT: usize = 2;
/// The maximum number of choices a poll is allowed to have
const MAX_CHOICE_COUNT: usize = 64;
/// The minimum number of bytes a choice description must have
const MIN_DESCRIPTION_LENGTH: usize = 1;
/// The maximum number of bytes a choice description is allowed to have
const MAX_DESCRIPTION_LENGTH: usize = 100;

#[derive(Debug)]
pub struct PollsModule {
    polls: HashMap<RoomKind, Poll>,
}

#[derive(Debug)]
pub enum PollsLoopback {
    StoppedByModerator,
    Expired,
    ChannelDropped,
}

impl From<ChannelDroppedError> for PollsLoopback {
    fn from(_: ChannelDroppedError) -> Self {
        Self::ChannelDropped
    }
}

/// Contains the state of a poll and a [`Sender`] to cancel it
#[derive(Debug)]
struct Poll {
    /// The state of the poll
    pub state: PollsState,

    /// The votes that were cast
    pub voted_choice_ids: HashMap<ParticipantId, BTreeSet<ChoiceId>>,

    /// Cancels the poll
    pub tx_cancel: Sender<PollsLoopback>,
}

impl Poll {
    /// The current result of the poll
    pub fn results(&self) -> Vec<Item> {
        let votes = self.voted_choice_ids.values().flatten();
        let mut results: HashMap<ChoiceId, u32> = self
            .state
            .choices
            .iter()
            .map(|choice| (choice.id, 0))
            .collect();

        for vote in votes {
            *results.entry(*vote).or_insert(0) += 1;
        }

        let mut results = results
            .into_iter()
            .map(|(id, count)| Item { id, count })
            .collect::<Vec<_>>();
        results.sort_by_key(|a| a.id);
        results
    }
}

impl SignalingModuleDescription for PollsModule {
    const MODULE_ID: ModuleId = POLLS_MODULE_ID;
    const DESCRIPTION: &'static str = "Handles meeting polls functionality";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];

    type Settings = NoSettings;
}

impl SignalingModule for PollsModule {
    const NAMESPACE: ModuleId = POLLS_MODULE_ID;

    type Incoming = PollsCommand;

    type Outgoing = PollsEvent;

    type Internal = NoOp;

    type Loopback = PollsLoopback;

    type JoinInfo = PollsState;

    type PeerJoinInfo = ();

    type Error = Error;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self {
            polls: HashMap::new(),
        })
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        _participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Error>> {
        let poll = self.polls.get(&ctx.room);

        if let Some(poll) = poll {
            Ok(ModuleJoinData {
                join_success: Some(poll.state.clone()),
                peer_events: PeerDataMap::default(),
                peer_data: PeerDataMap::default(),
            })
        } else {
            Ok(ModuleJoinData::default())
        }
    }

    #[allow(unused_variables)]
    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Error>> {
        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        _connection_id: ConnectionId,
        payload: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Error>> {
        match payload {
            PollsCommand::Start {
                topic,
                live,
                multiple_choice,
                choices,
                duration,
            } => self.start_poll(ctx, sender, topic, live, multiple_choice, choices, duration),
            PollsCommand::Vote(vote) => self.vote(ctx, sender, vote),
            PollsCommand::Finish { id } => self.finish_poll(ctx, sender, id),
        }
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Error>> {
        match event {
            // Everything is already handled when stopped by a moderator
            PollsLoopback::StoppedByModerator => {}
            PollsLoopback::Expired => {
                if let Some(poll) = self.polls.remove(&ctx.room) {
                    Self::send_results(ctx, &poll)?;
                }
            }
            PollsLoopback::ChannelDropped => {
                ctx.send_ws_message(
                    ctx.participants.in_room(ctx.room).ids(),
                    PollsEvent::Error(Error::Internal),
                )?;
            }
        }

        Ok(())
    }

    fn on_breakout_switch(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _old_room: RoomKind,
        _new_room: RoomKind,
    ) -> Result<ModuleSwitchData<Self>, SignalingModuleError<Error>> {
        let poll = self.polls.get(&ctx.room);

        let Some(poll) = poll else {
            return Ok(ModuleSwitchData::<Self>::new());
        };

        if poll.state.remaining().is_none() {
            return Ok(ModuleSwitchData::<Self>::new());
        }

        let connections = ctx
            .participant_state(participant_id)
            .with_context(|| format!("Participant '{participant_id}' does not have state"))?
            .connections();

        let switch_success = connections
            .map(|con| (con, Some(poll.state.clone())))
            .collect();
        Ok(ModuleSwitchData {
            switch_success,
            ..Default::default()
        })
    }

    fn on_breakout_closed(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<Error>> {
        self.polls.retain(|room, _| *room == RoomKind::Main);

        Ok(())
    }
}

impl PollsModule {
    fn is_running(&self, room: RoomKind) -> bool {
        self.polls.contains_key(&room)
    }

    #[allow(clippy::too_many_arguments)]
    fn start_poll(
        &mut self,
        ctx: &ModuleContext<'_, Self>,
        sender: ParticipantId,
        topic: String,
        live: bool,
        multiple_choice: bool,
        choices: Vec<String>,
        duration: Duration,
    ) -> Result<(), SignalingModuleError<Error>> {
        if !ctx.is_moderator(sender) {
            return Err(Error::InsufficientPermissions.into());
        }

        if self.is_running(ctx.room) {
            return Err(Error::StillRunning.into());
        }

        if duration > MAX_POLL_DURATION {
            return Err(Error::InvalidDuration {
                max_duration: MAX_POLL_DURATION,
            }
            .into());
        }

        if topic.len() > MAX_TOPIC_LENGTH {
            return Err(Error::InvalidTopicLength {
                max_length: MAX_TOPIC_LENGTH,
            }
            .into());
        }

        if !matches!(choices.len(), MIN_CHOICE_COUNT..=MAX_CHOICE_COUNT) {
            return Err(Error::InvalidChoiceCount {
                min_choice_count: MIN_CHOICE_COUNT,
                max_choice_count: MAX_CHOICE_COUNT,
            }
            .into());
        }

        if choices.iter().any(|content| {
            !matches!(
                content.len(),
                MIN_DESCRIPTION_LENGTH..=MAX_DESCRIPTION_LENGTH
            )
        }) {
            return Err(Error::InvalidChoiceDescriptionLength {
                min_length: MIN_DESCRIPTION_LENGTH,
                max_length: MAX_DESCRIPTION_LENGTH,
            }
            .into());
        }

        // Start a loopback that stops the poll when its duration is reached
        let tx_cancel = ctx.loopback_after(duration, || PollsLoopback::Expired);

        let choices: Vec<Choice> = choices
            .into_iter()
            .enumerate()
            .map(|(i, content)| Choice {
                id: ChoiceId::from(i as u32),
                content,
            })
            .collect();

        let id = PollId::generate();
        let state = PollsState {
            id,
            topic: topic.clone(),
            live,
            multiple_choice,
            choices: choices.clone(),
            started: ctx.timestamp,
            duration,
        };
        let poll = Poll {
            state,
            tx_cancel,
            voted_choice_ids: HashMap::new(),
        };
        self.polls.insert(ctx.room, poll);

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            PollsEvent::Started {
                id,
                topic,
                live,
                multiple_choice,
                choices,
                duration,
            },
        )?;

        Ok(())
    }

    fn vote(
        &mut self,
        ctx: &ModuleContext<'_, Self>,
        sender: ParticipantId,
        Vote { poll_id, choices }: Vote,
    ) -> Result<(), SignalingModuleError<Error>> {
        let Some(poll) = self
            .polls
            .get_mut(&ctx.room)
            .filter(|poll| poll.state.id == poll_id && !poll.state.is_expired())
        else {
            return Err(Error::InvalidPollId.into());
        };

        let choice_ids = choices.to_hash_set();
        if choice_ids.len() > 1 && !poll.state.multiple_choice {
            return Err(Error::MultipleChoicesNotAllowed.into());
        }

        let valid_choice_ids: BTreeSet<ChoiceId> =
            poll.state.choices.iter().map(|choice| choice.id).collect();
        if !choice_ids.is_subset(&valid_choice_ids) {
            return Err(Error::InvalidChoiceId.into());
        }

        poll.voted_choice_ids.insert(sender, choice_ids);

        if poll.state.live {
            ctx.send_ws_message(
                ctx.participants.in_room(ctx.room).connected().ids(),
                PollsEvent::LiveUpdate(Results {
                    id: poll_id,
                    results: poll.results(),
                }),
            )?;
        }

        Ok(())
    }

    fn finish_poll(
        &mut self,
        ctx: &ModuleContext<'_, Self>,
        sender: ParticipantId,
        poll_id: PollId,
    ) -> Result<(), SignalingModuleError<Error>> {
        if !ctx.is_moderator(sender) {
            return Err(Error::InsufficientPermissions.into());
        }

        let entry = self.polls.entry(ctx.room);
        let Entry::Occupied(occupied) = entry else {
            return Err(Error::InvalidPollId.into());
        };
        let poll = occupied.get();
        if poll.state.id != poll_id {
            return Err(Error::InvalidPollId.into());
        }

        let poll = occupied.remove();
        Self::send_results(ctx, &poll)?;

        // Cancel the running poll
        if poll
            .tx_cancel
            .send(PollsLoopback::StoppedByModerator)
            .is_err()
        {
            tracing::debug!("Poll cancel sender has been dropped");
        }

        Ok(())
    }

    /// Sends the results of a poll to all participants in the room
    fn send_results(ctx: &ModuleContext<'_, Self>, poll: &Poll) -> Result<(), FatalError> {
        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            PollsEvent::Done(Results {
                id: poll.state.id,
                results: poll.results(),
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::signaling_module::SignalingModuleDescription as _;

    use crate::PollsModule;

    #[test]
    fn serialize_example_config() {
        let produced = toml::to_string_pretty(&PollsModule::example_config()).unwrap();

        assert_snapshot!(produced, @"[polls]");
    }
}
