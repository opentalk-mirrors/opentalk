// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::borrow::Cow;

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Duration, Utc};
use chrono_tz::Tz;
use ics::{
    Event, ICalendar,
    components::Property,
    escape_text, parameters,
    properties::{
        Attendee, Created, Description, ExDate, LastModified, Method, Organizer, RDate, RRule,
        Sequence, Status, Summary,
    },
};
use ics_chrono_tz::ToIcsTimeZone;
use opentalk_mail_worker_protocol::v1::{self, EventException, Time};
use opentalk_types_common::shared_folders::SharedFolder;
use uuid::Uuid;

#[derive(Clone, Copy)]
pub enum Invitee<'a> {
    WithName { email: &'a str, name: &'a str },
    WithoutName(&'a str),
}

pub enum EventStatus {
    Created,
    Updated,
    Cancelled,
}

/// The Event type that is used when creating an ICS event object.
///
/// The type is similar to the [`v1::Event`] but the `start_time` and `end_time` fields are mandatory
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct IcsCompatibleEvent {
    pub id: Uuid,
    pub name: String,
    pub created_at: Time,
    pub start_time: Time,
    pub end_time: Time,
    pub rrule: Option<String>,
    pub description: String,
    pub room: v1::Room,
    pub call_in: Option<v1::CallIn>,
    pub revision: i32,
    pub shared_folder: Option<SharedFolder>,
}

impl TryFrom<v1::Event> for IcsCompatibleEvent {
    type Error = anyhow::Error;

    fn try_from(event: v1::Event) -> std::result::Result<Self, Self::Error> {
        Ok(IcsCompatibleEvent {
            id: event.id,
            name: event.name.to_string(),
            created_at: event.created_at,
            start_time: event
                .start_time
                .ok_or_else(|| anyhow!("missing start time"))?,
            end_time: event.end_time.ok_or_else(|| anyhow!("missing end time"))?,
            rrule: event.rrule,
            description: event.description.to_string(),
            room: event.room,
            call_in: event.call_in,
            revision: event.revision,
            shared_folder: event.shared_folder,
        })
    }
}

pub(crate) fn create_ics_v1(
    inviter: &v1::RegisteredUser,
    event: &v1::Event,
    exception: Option<&v1::EventException>,
    invitee: Invitee,
    description: &str,
    status: EventStatus,
) -> Result<Option<Vec<u8>>> {
    let event = if let Ok(event) = IcsCompatibleEvent::try_from(event.clone()) {
        event
    } else {
        return Ok(None);
    };

    let mut calendar = ICalendar::new("2.0", "-//OpenTalk GmbH//NONSGML smtp-mailer v1.0//EN");

    let start_tz: Tz = event
        .start_time
        .timezone
        .parse()
        .ok()
        .context("start timezone")?;

    let end_tz: Tz = event
        .end_time
        .timezone
        .parse()
        .ok()
        .context("end timezone")?;

    calendar.add_timezone(ToIcsTimeZone::to_latest_ics_timezone(&start_tz));

    if start_tz != end_tz {
        calendar.add_timezone(ToIcsTimeZone::to_latest_ics_timezone(&end_tz));
    }

    match status {
        EventStatus::Created | EventStatus::Updated => {
            // if the ics file is created for cancelling an event exception, the ics Method should be `CANCEL`
            if let Some(v1::EventExceptionKind::Canceled) = exception.map(|e| &e.kind) {
                calendar.push(Method::new("CANCEL"));
            } else {
                calendar.push(Method::new("REQUEST"));
            }
        }
        EventStatus::Cancelled => {
            calendar.push(Method::new("CANCEL"));
        }
    }
    let mut buf = Vec::new();

    if let Some(exception) = exception {
        let exception_event_obj =
            create_exception_event_object(&event, description, exception, inviter, invitee);

        calendar.add_event(exception_event_obj);

        calendar.write(&mut buf)?;
    } else {
        let event_obj = create_event_object(&event, description, inviter, invitee, &status);

        calendar.add_event(event_obj);

        calendar.write(&mut buf)?;
    }

    Ok(Some(buf))
}

fn create_event_object<'a>(
    event: &'a IcsCompatibleEvent,
    description: &'a str,
    inviter: &'a v1::RegisteredUser,
    invitee: Invitee<'a>,
    status: &EventStatus,
) -> Event<'a> {
    let mut event_obj = Event::new(
        event.id.to_string(),
        Utc::now().format("%Y%m%dT%H%M%SZ").to_string(),
    );

    // add properties
    event_obj.push(Summary::new(escape_text(event.name.clone())));

    event_obj.push(Description::new(escape_text(description)));

    let created_at_tz = event.created_at.timezone.parse().unwrap_or(Tz::UTC);
    let created_at = event.created_at.time.with_timezone(&created_at_tz);

    event_obj.push(Created::new(
        created_at.format("%Y%m%dT%H%M%SZ").to_string(),
    ));

    event_obj.push(LastModified::new(
        Utc::now().format("%Y%m%dT%H%M%SZ").to_string(),
    ));

    event_obj.push(Sequence::new(event.revision.to_string()));

    match status {
        EventStatus::Created | EventStatus::Updated => {
            event_obj.push(Status::confirmed());
        }
        EventStatus::Cancelled => {
            event_obj.push(Status::cancelled());
        }
    }

    let mut organizer_property = Organizer::new(format!("mailto:{}", inviter.email.as_ref()));
    organizer_property
        .append(parameters!("CN" => format!("{} {}", inviter.first_name, inviter.last_name)));
    event_obj.push(organizer_property);

    match invitee {
        Invitee::WithName { email, name } => {
            let mut attendee_property = Attendee::new(format!("mailto:{email}"));

            attendee_property.append(parameters!(
                "CUTYPE" => "INDIVIDUAL";
                "ROLE" => "REQ-PARTICIPANT";
                "PARTSTAT" => "NEEDS-ACTION";
                "CN" => name;
                "RSVP" => "TRUE";
                "X-NUM-GUESTS" => "0"
            ));

            event_obj.push(attendee_property);
        }
        Invitee::WithoutName(email) => {
            let mut attendee_property = Attendee::new(format!("mailto:{email}"));

            attendee_property.append(parameters!(
                "CUTYPE" => "INDIVIDUAL";
                "ROLE" => "REQ-PARTICIPANT";
                "PARTSTAT" => "NEEDS-ACTION";
                "CN" => email;
                "RSVP" => "TRUE";
                "X-NUM-GUESTS" => "0"
            ));

            event_obj.push(attendee_property);
        }
    }

    let dt_start =
        create_time_property("DTSTART", event.start_time.time, &event.start_time.timezone);

    event_obj.push(dt_start.clone());

    if let Some(rrule) = &event.rrule {
        // Todo: The Display implementation of the rrule crate is not really usable.
        // Therefore we rely on the verification of the web-api

        for sigment in rrule.split('\n') {
            if sigment.starts_with("RRULE:") {
                let rrule_property = RRule::new(sigment.trim_start_matches("RRULE:"));
                event_obj.push(rrule_property);
            } else if sigment.starts_with("EXDATE:") {
                let exdate_property = ExDate::new(sigment.trim_start_matches("EXDATE:"));
                event_obj.push(exdate_property);
            } else if sigment.starts_with("RDATE:") {
                let rdate_property = RDate::new(sigment.trim_start_matches("RDATE:"));
                event_obj.push(rdate_property);
            }
        }
    }

    event_obj.push(create_time_property(
        "DTEND",
        event.end_time.time,
        &event.end_time.timezone,
    ));

    event_obj
}

/// Create the ICS event object for an event instance update
fn create_exception_event_object<'a>(
    event: &'a IcsCompatibleEvent,
    description: &'a str,
    exception: &'a EventException,
    inviter: &'a v1::RegisteredUser,
    invitee: Invitee<'a>,
) -> Event<'a> {
    let now = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();

    let mut event_obj = Event::new(
        event.id.to_string(),
        Utc::now().format("%Y%m%dT%H%M%SZ").to_string(),
    );

    let recurrence_id = create_time_property(
        "RECURRENCE-ID",
        exception.exception_date.time,
        &exception.exception_date.timezone,
    );

    event_obj.push(recurrence_id);

    let created_at_tz = event.created_at.timezone.parse().unwrap_or(Tz::UTC);
    let created_at = event.created_at.time.with_timezone(&created_at_tz);

    event_obj.push(Created::new(
        created_at.format("%Y%m%dT%H%M%SZ").to_string(),
    ));

    event_obj.push(LastModified::new(now));

    match exception.kind {
        v1::EventExceptionKind::Modified => {
            event_obj.push(Status::confirmed());
        }
        v1::EventExceptionKind::Canceled => {
            event_obj.push(Status::cancelled());
        }
    }

    // use the new instance title or fallback to the original
    let title = exception
        .title
        .as_ref()
        .map(|e| e.to_string())
        .unwrap_or_else(|| event.name.to_string());
    event_obj.push(Summary::new(escape_text(title)));

    // use the new instance description or fallback to the original
    let description: Cow<'a, str> = exception
        .description
        .as_ref()
        .map(|d| d.to_string())
        .map(Into::into)
        .unwrap_or_else(|| description.into());

    // FIXME: description changes would remove the meeting link and data privacy disclaimer
    event_obj.push(Description::new(escape_text(description)));

    event_obj.push(Sequence::new((event.revision).to_string()));

    let mut organizer_property = Organizer::new(format!("mailto:{}", inviter.email.as_ref()));
    organizer_property
        .append(parameters!("CN" => format!("{} {}", inviter.first_name, inviter.last_name)));
    event_obj.push(organizer_property);

    match invitee {
        Invitee::WithName { email, name } => {
            let mut attendee_property = Attendee::new(format!("mailto:{email}"));

            attendee_property.append(parameters!(
                "CUTYPE" => "INDIVIDUAL";
                "ROLE" => "REQ-PARTICIPANT";
                "PARTSTAT" => "NEEDS-ACTION";
                "CN" => name;
                "RSVP" => "TRUE";
                "X-NUM-GUESTS" => "0"
            ));

            event_obj.push(attendee_property);
        }
        Invitee::WithoutName(email) => {
            let mut attendee_property = Attendee::new(format!("mailto:{email}"));

            attendee_property.append(parameters!(
                "CUTYPE" => "INDIVIDUAL";
                "ROLE" => "REQ-PARTICIPANT";
                "PARTSTAT" => "NEEDS-ACTION";
                "CN" => email;
                "RSVP" => "TRUE";
                "X-NUM-GUESTS" => "0"
            ));

            event_obj.push(attendee_property);
        }
    }

    // FIXME: handle all_day events, these are not handled at all from the SMTP mailer at the moment.
    // if let Some(is_all_day) = exception.is_all_day

    let dt_start = exception
        .starts_at
        .as_ref()
        .unwrap_or(&exception.exception_date);

    event_obj.push(create_time_property(
        "DTSTART",
        dt_start.time,
        &dt_start.timezone,
    ));

    let end_time_property = match &exception.ends_at {
        Some(end_time) => create_time_property("DTEND", end_time.time, &end_time.timezone),
        None => {
            let duration = calculate_meeting_duration(&event.start_time, &event.end_time);

            let exception_tz = dt_start.timezone.parse().unwrap_or(Tz::UTC);
            let exception_start = dt_start.time.with_timezone(&exception_tz);

            let exception_end = exception_start + duration;

            // create the DTEND property with the same timezone as DTSTART
            create_time_property(
                "DTEND",
                exception_end.with_timezone(&Utc),
                &dt_start.timezone,
            )
        }
    };

    event_obj.push(end_time_property);

    event_obj
}

fn create_time_property<'a>(
    property_name: &'a str,
    time: DateTime<Utc>,
    timezone: &'a str,
) -> Property<'a> {
    match timezone.parse::<Tz>().ok() {
        Some(Tz::UTC) | None => {
            Property::new(property_name, time.format("%Y%m%dT%H%M%SZ").to_string())
        }
        Some(tz) => {
            let start_time = time.with_timezone(&tz);
            let mut prop = Property::new(
                property_name,
                start_time.format("%Y%m%dT%H%M%S").to_string(),
            );
            prop.append(parameters!("TZID" => timezone));
            prop
        }
    }
}

/// Takes the original start and end time to calculate the original duration of the meeting.
///
/// The duration is then applied to the exceptions start time, considering all configured timezones
fn calculate_meeting_duration(start: &Time, end: &Time) -> Duration {
    let start = start
        .time
        .with_timezone(&start.timezone.parse().unwrap_or(Tz::UTC));
    let end = end
        .time
        .with_timezone(&end.timezone.parse().unwrap_or(Tz::UTC));

    end.signed_duration_since(start).max(Duration::zero())
}
#[cfg(test)]
mod test {
    use std::str::FromStr as _;

    use chrono::{TimeZone, Utc};
    use opentalk_mail_worker_protocol::v1::{CallIn, Event, RegisteredUser, Room, Time};
    use opentalk_types_common::{
        rooms::RoomPassword,
        streaming::{RoomStreamingTarget, StreamingKey, StreamingTarget, StreamingTargetKind},
        users::{Language, UserTitle},
    };
    use uuid::Uuid;

    use super::{Invitee, create_ics_v1};
    use crate::ics::EventStatus;

    #[test]
    fn test_ics() {
        let user = RegisteredUser {
            email: "klaus@example.org".into(),
            title: UserTitle::from_str("Dr.").expect("Dr. is a valid UserTitle"),
            first_name: "Klaus".to_owned(),
            last_name: "Doktor".to_owned(),
            language: Language::from_str("de_DE").expect("de_DE is valid language"),
        };

        let event = Event {
            id: Uuid::from_u128(2),
            name: "Test".parse().expect("Example must be valid"),
            created_at: Time {
                time: Utc.with_ymd_and_hms(2022, 6, 20, 0, 0, 0).unwrap(),
                timezone: "UTC".to_owned(),
            },
            start_time: Some(Time {
                time: Utc.with_ymd_and_hms(2022, 6, 20, 0, 0, 0).unwrap(),
                timezone: "Europe/Berlin".to_owned(),
            }),
            end_time: Some(Time {
                time: Utc.with_ymd_and_hms(2022, 6, 20, 10, 0, 0).unwrap(),
                timezone: "Europe/Berlin".to_owned(),
            }),
            rrule: Some("RRULE:FREQ=WEEKLY;COUNT=30;INTERVAL=1".to_owned()),
            call_in: Some(CallIn {
                sip_tel: "+4980011880".to_owned(),
                sip_id: "0123456789".to_owned(),
                sip_password: "555NASE".to_owned(),
            }),
            description: "Very descriptive".parse().expect("Example must be valid"),
            room: Room {
                id: Uuid::from_u128(3),
                alias: None,
                password: Some(RoomPassword::from_str("ddd").expect("Invalid room password")),
            },
            revision: 0,
            shared_folder: None,
            adhoc_retention_seconds: Some(86400),
            streaming_targets: vec![
                RoomStreamingTarget {
                    id: Uuid::new_v4().into(),
                    streaming_target: StreamingTarget {
                        name: "streaming service 1".to_string(),
                        kind: StreamingTargetKind::Custom {
                            streaming_endpoint: "https://stream-a.example.com"
                                .parse()
                                .expect("This is a valid URL!"),
                            streaming_key: StreamingKey::from("value".to_string()),
                            public_url: "https://example.com/4385f873-6077-4d2c-8b46-dbec62029ac6"
                                .parse()
                                .expect("This is a valid URL!"),
                        },
                    },
                },
                RoomStreamingTarget {
                    id: Uuid::new_v4().into(),
                    streaming_target: StreamingTarget {
                        name: "streaming service 2".to_string(),
                        kind: StreamingTargetKind::Custom {
                            streaming_endpoint: "https://stream-b.example.com"
                                .parse()
                                .expect("This is a valid URL!"),
                            streaming_key: StreamingKey::from("value".to_string()),
                            public_url: "https://example.com/f544bef8-aa33-475b-8664-fa17c5ca7f83"
                                .parse()
                                .expect("This is a valid URL!"),
                        },
                    },
                },
            ],
        };

        let invitee = Invitee::WithName {
            email: "g@example.org",
            name: "G. G.",
        };

        let ics = create_ics_v1(
            &user,
            &event,
            None,
            invitee,
            &event.description.to_string(),
            EventStatus::Created,
        )
        .expect("Failed to create ics file")
        .expect("No ics file for this event");

        let ics_str = String::from_utf8_lossy(&ics);
        let to_test = ics_str.lines().collect::<Vec<_>>();

        assert!(to_test.contains(&"BEGIN:VEVENT"));

        // lines that are longer than 75 octets are split with a CRLF followed by a whitespace (see https://datatracker.ietf.org/doc/html/rfc5545#section-3.1)
        assert!(to_test.contains(
            &"ATTENDEE;CN=G. G.;CUTYPE=INDIVIDUAL;PARTSTAT=NEEDS-ACTION;ROLE=REQ-PARTICIP"
        ));
        assert!(to_test.contains(&" ANT;RSVP=TRUE;X-NUM-GUESTS=0:mailto:g@example.org"));

        assert!(to_test.contains(&"ORGANIZER;CN=Klaus Doktor:mailto:klaus@example.org"));
        assert!(to_test.contains(&"SUMMARY:Test"));
        assert!(to_test.contains(&"DESCRIPTION:Very descriptive"));
        assert!(to_test.contains(&"DTSTART;TZID=Europe/Berlin:20220620T020000"));
        assert!(to_test.contains(&"DTEND;TZID=Europe/Berlin:20220620T120000"));
        assert!(to_test.contains(&"RRULE:FREQ=WEEKLY;COUNT=30;INTERVAL=1"));
    }
}
