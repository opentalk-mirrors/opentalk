// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use futures_core::Stream;
use futures_util::{StreamExt, TryStreamExt, pin_mut};
use opentalk_controller_utils::CaptureApiError;
use opentalk_types_api_v1::{
    events::{EventOrException, EventOrInstance, EventResource, GetEventsCursorData},
    pagination::Cursor,
};
use opentalk_types_common::{order::Ordering, pagination::PageSize};

// Keep pagination policy here so resource loading can stay independent of future changes.
pub(super) struct EventsPage<T> {
    items: Vec<T>,
    after: Option<String>,
}

impl EventsPage<EventOrException> {
    pub(super) async fn from_events(
        stream: impl Stream<Item = Result<EventOrException, CaptureApiError>>,
        per_page: Option<PageSize>,
        ordering: Ordering,
    ) -> Result<Self, CaptureApiError> {
        let limit: usize = per_page.unwrap_or_default().into();
        let items: Vec<_> = stream.take(limit).try_collect().await?;

        // Keep the legacy next-page link until the events pagination contract is revised.
        let cursor = items.last().map(|item| match item {
            EventOrException::Event(event) => event_cursor(event),
            EventOrException::Exception(exception) => GetEventsCursorData {
                event_id: exception.recurring_event_id,
                event_created_at: exception.created_at,
                event_starts_at: exception.starts_at.map(Into::into),
                instance_id: Some(exception.instance_id),
            },
        });

        Ok(Self::new(items, ordering, cursor))
    }
}

impl EventsPage<EventOrInstance> {
    pub(super) async fn from_instances(
        stream: impl Stream<Item = Result<EventOrInstance, CaptureApiError>>,
        per_page: Option<PageSize>,
        ordering: Ordering,
    ) -> Result<Self, CaptureApiError> {
        let limit: usize = per_page.unwrap_or_default().into();
        let (items, has_more_items) = collect_page(stream, limit).await?;

        let cursor = if has_more_items {
            items.last().map(|item| match item {
                EventOrInstance::Event(event) => event_cursor(event),
                EventOrInstance::Instance(instance) => GetEventsCursorData {
                    event_id: instance.recurring_event_id,
                    event_created_at: instance.created_at,
                    event_starts_at: Some(instance.starts_at.into()),
                    instance_id: Some(instance.instance_id),
                },
            })
        } else {
            None
        };

        Ok(Self::new(items, ordering, cursor))
    }
}

impl<T> EventsPage<T> {
    fn new(mut items: Vec<T>, ordering: Ordering, cursor: Option<GetEventsCursorData>) -> Self {
        // Pagination follows the stream order, even when the response order differs.
        let after = cursor.map(|data| Cursor(data).to_base64());

        if ordering == Ordering::Descending {
            // Preserve frontend compatibility until ordering is exposed through the API.
            items.reverse();
        }

        Self { items, after }
    }

    pub(super) fn into_parts(self) -> (Vec<T>, Option<String>, Option<String>) {
        (self.items, None, self.after)
    }
}

fn event_cursor(event: &EventResource) -> GetEventsCursorData {
    GetEventsCursorData {
        event_id: event.id,
        event_created_at: event.created_at,
        event_starts_at: event.date.starts_at().cloned().map(Into::into),
        instance_id: None,
    }
}

async fn collect_page<T, E, S>(stream: S, limit: usize) -> Result<(Vec<T>, bool), E>
where
    S: Stream<Item = Result<T, E>>,
{
    pin_mut!(stream);
    let mut items = Vec::with_capacity(limit);
    for _ in 0..limit {
        let Some(result) = stream.next().await else {
            return Ok((items, false));
        };
        items.push(result?);
    }
    let has_more = stream.next().await.transpose()?.is_some();
    Ok((items, has_more))
}

#[cfg(test)]
mod tests {
    use opentalk_types_api_v1::events::{EventExceptionResource, EventInstance};
    use opentalk_types_common::{events::EventId, utils::ExampleData};

    use super::*;

    #[test]
    fn collect_page_empty_stream() {
        let stream = futures_util::stream::iter(Vec::<Result<i32, &'static str>>::new());
        let (items, has_more) = futures::executor::block_on(collect_page(stream, 5)).expect("ok");
        assert!(items.is_empty());
        assert!(!has_more);
    }

    #[test]
    fn collect_page_fewer_than_limit() {
        let stream = futures_util::stream::iter(vec![Ok::<_, &'static str>(1), Ok(2)]);
        let (items, has_more) = futures::executor::block_on(collect_page(stream, 5)).expect("ok");
        assert_eq!(items, vec![1, 2]);
        assert!(!has_more);
    }

    #[test]
    fn collect_page_exactly_limit() {
        let stream = futures_util::stream::iter(vec![Ok::<_, &'static str>(1), Ok(2), Ok(3)]);
        let (items, has_more) = futures::executor::block_on(collect_page(stream, 3)).expect("ok");
        assert_eq!(items, vec![1, 2, 3]);
        assert!(
            !has_more,
            "no cursor must be emitted when exactly `limit` items exist"
        );
    }

    #[test]
    fn collect_page_more_than_limit() {
        let stream =
            futures_util::stream::iter(vec![Ok::<_, &'static str>(1), Ok(2), Ok(3), Ok(4)]);
        let (items, has_more) = futures::executor::block_on(collect_page(stream, 3)).expect("ok");
        assert_eq!(items, vec![1, 2, 3]);
        assert!(has_more);
    }

    #[test]
    fn collect_page_zero_limit() {
        let stream = futures_util::stream::iter(vec![Ok::<_, &'static str>(1)]);
        let (items, has_more) = futures::executor::block_on(collect_page(stream, 0)).expect("ok");
        assert!(items.is_empty());
        assert!(has_more);
    }

    #[test]
    fn collect_page_propagates_error_within_limit() {
        let stream = futures_util::stream::iter(vec![Ok(1), Err::<i32, _>("boom"), Ok(3)]);
        let err = futures::executor::block_on(collect_page(stream, 5));
        assert_eq!(err, Err("boom"));
    }

    #[test]
    fn collect_page_propagates_error_at_peek() {
        let stream = futures_util::stream::iter(vec![Ok(1), Ok(2), Err::<i32, _>("peek-err")]);
        let err = futures::executor::block_on(collect_page(stream, 2));
        assert_eq!(err, Err("peek-err"));
    }

    fn resource_stream<T>(items: Vec<T>) -> impl Stream<Item = Result<T, CaptureApiError>> {
        futures_util::stream::iter(items.into_iter().map(Ok))
    }

    fn event(id: u128) -> EventResource {
        EventResource {
            id: EventId::from_u128(id),
            ..EventResource::example_data()
        }
    }

    fn decode_cursor(after: Option<String>) -> GetEventsCursorData {
        Cursor::from_base64(&after.unwrap()).unwrap().0
    }

    #[tokio::test]
    async fn events_page_keeps_ascending_order_and_builds_event_cursor() {
        let first = event(1);
        let last = event(2);
        let expected = vec![
            EventOrException::Event(first),
            EventOrException::Event(last.clone()),
        ];
        let page =
            EventsPage::from_events(resource_stream(expected.clone()), None, Ordering::Ascending)
                .await
                .unwrap();
        let (items, before, after) = page.into_parts();

        assert_eq!(items, expected);
        assert_eq!(before, None);
        assert_eq!(
            decode_cursor(after),
            GetEventsCursorData {
                event_id: last.id,
                event_created_at: last.created_at,
                event_starts_at: last.date.starts_at().cloned().map(Into::into),
                instance_id: None,
            }
        );
    }

    #[tokio::test]
    async fn events_page_builds_exception_cursor_before_reversing() {
        let first = EventOrException::Event(event(2));
        let exception = EventExceptionResource {
            starts_at: None,
            ..EventExceptionResource::example_data()
        };
        let last = EventOrException::Exception(exception.clone());
        let page = EventsPage::from_events(
            resource_stream(vec![first.clone(), last.clone()]),
            None,
            Ordering::Descending,
        )
        .await
        .unwrap();
        let (items, before, after) = page.into_parts();

        assert_eq!(items, vec![last, first]);
        assert_eq!(before, None);
        assert_eq!(
            decode_cursor(after),
            GetEventsCursorData {
                event_id: exception.recurring_event_id,
                event_created_at: exception.created_at,
                event_starts_at: None,
                instance_id: Some(exception.instance_id),
            }
        );
    }

    #[tokio::test]
    async fn instances_page_builds_instance_cursor_before_reversing() {
        let first = EventOrInstance::Event(event(2));
        let instance = EventInstance::example_data();
        let last = EventOrInstance::Instance(instance.clone());
        let page = EventsPage::from_instances(
            resource_stream(vec![
                first.clone(),
                last.clone(),
                EventOrInstance::Event(event(0)),
            ]),
            Some(PageSize::from_i64_clamped(2)),
            Ordering::Descending,
        )
        .await
        .unwrap();
        let (items, before, after) = page.into_parts();

        assert_eq!(items, vec![last, first]);
        assert_eq!(before, None);
        assert_eq!(
            decode_cursor(after),
            GetEventsCursorData {
                event_id: instance.recurring_event_id,
                event_created_at: instance.created_at,
                event_starts_at: Some(instance.starts_at.into()),
                instance_id: Some(instance.instance_id),
            }
        );
    }

    #[tokio::test]
    async fn instances_page_builds_event_cursor() {
        let event = event(1);
        let page = EventsPage::from_instances(
            resource_stream(vec![
                EventOrInstance::Event(event.clone()),
                EventOrInstance::example_data(),
            ]),
            Some(PageSize::from_i64_clamped(1)),
            Ordering::Ascending,
        )
        .await
        .unwrap();
        let (_, _, after) = page.into_parts();

        assert_eq!(
            decode_cursor(after),
            GetEventsCursorData {
                event_id: event.id,
                event_created_at: event.created_at,
                event_starts_at: event.date.starts_at().cloned().map(Into::into),
                instance_id: None,
            }
        );
    }

    #[tokio::test]
    async fn final_instances_page_has_no_cursor() {
        let first = EventOrInstance::Event(event(2));
        let last = EventOrInstance::Event(event(1));
        let page = EventsPage::from_instances(
            resource_stream(vec![first.clone(), last.clone()]),
            Some(PageSize::from_i64_clamped(2)),
            Ordering::Descending,
        )
        .await
        .unwrap();
        let (items, before, after) = page.into_parts();

        assert_eq!(items, vec![last, first]);
        assert_eq!(before, None);
        assert_eq!(after, None);
    }

    #[tokio::test]
    async fn empty_pages_have_no_cursor() {
        let events =
            EventsPage::from_events(resource_stream(Vec::new()), None, Ordering::Ascending)
                .await
                .unwrap();
        assert_eq!(events.into_parts(), (Vec::new(), None, None));

        let instances =
            EventsPage::from_instances(resource_stream(Vec::new()), None, Ordering::Descending)
                .await
                .unwrap();
        assert_eq!(instances.into_parts(), (Vec::new(), None, None));
    }

    #[tokio::test]
    async fn events_page_does_not_poll_past_its_limit() {
        let first = EventOrException::Event(event(1));
        let stream = futures_util::stream::iter(vec![
            Ok(first.clone()),
            Err(opentalk_types_api_v1::error::ApiError::internal().into()),
        ]);
        let page = EventsPage::from_events(
            stream,
            Some(PageSize::from_i64_clamped(1)),
            Ordering::Ascending,
        )
        .await
        .unwrap();
        let (items, _, after) = page.into_parts();

        assert_eq!(items, vec![first]);
        assert_eq!(decode_cursor(after).event_id, EventId::from_u128(1));
    }

    #[tokio::test]
    async fn both_endpoints_apply_the_default_page_size() {
        let limit = usize::from(PageSize::default());
        let events = (0..=limit)
            .map(|id| EventOrException::Event(event(id as u128)))
            .collect();
        let page = EventsPage::from_events(resource_stream(events), None, Ordering::Ascending)
            .await
            .unwrap();
        let (items, _, after) = page.into_parts();

        assert_eq!(items.len(), limit);
        assert_eq!(
            decode_cursor(after).event_id,
            EventId::from_u128((limit - 1) as u128)
        );

        let instances = (0..=limit)
            .map(|id| EventOrInstance::Event(event(id as u128)))
            .collect();
        let page =
            EventsPage::from_instances(resource_stream(instances), None, Ordering::Ascending)
                .await
                .unwrap();
        let (items, _, after) = page.into_parts();

        assert_eq!(items.len(), limit);
        assert_eq!(
            decode_cursor(after).event_id,
            EventId::from_u128((limit - 1) as u128)
        );
    }
}
