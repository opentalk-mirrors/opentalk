// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{MediaEvent, RtpMpscSource};
use anyhow::{Context, Result};
use bitstream_io::{BigEndian, BitRead, BitReader};
use ezk::{NextEventIsCancelSafe, Source, SourceEvent};
use ezk_rtp::{Rtp, RtpConfig, RtpConfigRange};
use std::io::Cursor;
use tokio::sync::mpsc;

pub(super) struct DtmfFilter {
    pub(super) source: RtpMpscSource,

    pub(super) media_pt: u8,

    // (dtmf payload, dtmf event sender)
    pub(super) dtmf: Option<(u8, mpsc::UnboundedSender<MediaEvent>)>,
    pub(super) last_dtmf_timestamp: Option<u32>,
}

impl NextEventIsCancelSafe for DtmfFilter {}

impl Source for DtmfFilter {
    type MediaType = Rtp;

    async fn capabilities(
        &mut self,
    ) -> ezk::Result<Vec<<Self::MediaType as ezk::MediaType>::ConfigRange>> {
        self.source.capabilities().await
    }

    async fn negotiate_config(&mut self, available: Vec<RtpConfigRange>) -> ezk::Result<RtpConfig> {
        self.source.negotiate_config(available).await
    }

    async fn next_event(&mut self) -> ezk::Result<SourceEvent<Rtp>> {
        loop {
            let frame = match self.source.next_event().await? {
                SourceEvent::Frame(frame) => frame,
                SourceEvent::EndOfData => return Ok(SourceEvent::EndOfData),
                SourceEvent::RenegotiationNeeded => return Ok(SourceEvent::RenegotiationNeeded),
            };

            let packet = frame.data().get();

            if packet.payload_type() == self.media_pt {
                return Ok(SourceEvent::Frame(frame));
            }

            if let Some((pt, ref tx)) = self.dtmf {
                if packet.payload_type() != pt {
                    continue;
                }

                let event = match read_dtmf_event(packet.payload()) {
                    Ok(event) => event,
                    Err(e) => {
                        log::warn!("failed to parse dtmf event payload, {e:?}");
                        continue;
                    }
                };

                if self.last_dtmf_timestamp < Some(packet.timestamp()) {
                    self.last_dtmf_timestamp = Some(packet.timestamp());

                    log::trace!("Read DTMF event digit={event}");

                    if let Err(e) = tx.send(MediaEvent::DtmfDigit(event)) {
                        return Err(ezk::Error::new(ezk::ErrorKind::Other, e));
                    }
                }
            }
        }
    }
}

fn read_dtmf_event(payload: &[u8]) -> Result<u8> {
    let mut reader = BitReader::endian(Cursor::new(payload), BigEndian);

    let event: u8 = reader.read::<8, _>().context("Failed to read event")?;
    let _is_end: bool = reader.read_bit().context("Failed to read is_end")?;
    reader.skip(1).context("Failed to skip 1 bit")?;
    let _volume: u8 = reader.read::<6, _>().context("Failed to read volume")?;
    let _duration: u16 = reader.read::<16, _>().context("Failed to read duration")?;

    Ok(event)
}
