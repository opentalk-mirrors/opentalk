// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::HandleEvent;
use anyhow::{Context, Result};
use gio::glib::{Value, WeakRef};
use gio::prelude::*;
use gst::prelude::*;
use gst::traits::{ElementExt, GstBinExt, PadExt};
use tokio::sync::mpsc;
use uuid::Uuid;

/// Creates a closure which is called by webrtcbin when it's ready to negotitate via SDP.
// TODO: This currently will always parse the initial offer, ideally this should request a new offer via signaling
pub(super) fn webrtcbin_on_negotation_needed(
    participant_id: Uuid,
    task_tx: mpsc::UnboundedSender<HandleEvent>,
    offer: String,
) -> impl Fn(&[Value]) -> Option<Value> {
    move |values| {
        let webrtcbin = values[0]
            .get::<gst::Element>()
            .expect("values[0] of on-negotiation-needed is a gst::Element");

        if let Err(e) =
            try_webrtcbin_on_negotiation_needed(webrtcbin, participant_id, task_tx.clone(), &offer)
        {
            log::error!("Failed to handle subscriber-webrtc's on-negotiation-needed event, {e:?}");
        }

        None
    }
}

fn try_webrtcbin_on_negotiation_needed(
    webrtcbin: gst::Element,
    participant_id: Uuid,
    task_tx: mpsc::UnboundedSender<HandleEvent>,
    offer: &str,
) -> Result<()> {
    // Parse the SDP text into a gstreamer WebRTCSessionDescription
    let offer = gst_sdp::SDPMessage::parse_buffer(offer.as_bytes())
        .context("Failed to parse SDP offer to gst::SDPMessage")?;
    let offer = gst_webrtc::WebRTCSessionDescription::new(gst_webrtc::WebRTCSDPType::Offer, offer);

    // Set the offer as "remote" description to respond with a local description
    log::debug!("settings remote description to \n{}", offer.sdp());
    webrtcbin.emit_by_name::<()>("set-remote-description", &[&offer, &None::<gst::Promise>]);

    // Create callback to receive the SDP response from the webrtcbin
    let webrtcbin_weak = webrtcbin.downgrade();
    let promise = gst::Promise::with_change_func(move |answer| {
        let Some(webrtcbin) = webrtcbin_weak.upgrade() else {
            return;
        };

        // Extract the SDP Answer from the many layers of wrapper types
        let Ok(Some(structure)) = answer else {
            log::error!("Failed to create SDP Answer, {:?}", answer);
            return;
        };
        let Ok(answer) = structure.get::<gst_webrtc::WebRTCSessionDescription>("answer") else {
            return;
        };

        // Set our SDP response as local SDP
        log::debug!("setting local description to\n{}", answer.sdp());
        webrtcbin.emit_by_name::<()>("set-local-description", &[&answer, &None::<gst::Promise>]);

        // Send the SDP answer back to the signaling task
        let _ = task_tx.send(HandleEvent::SubscribeAnswer {
            uuid: participant_id,
            answer: answer.sdp().to_string(),
        });
    });

    webrtcbin.emit_by_name::<()>("create-answer", &[&None::<gst::Structure>, &promise]);

    Ok(())
}

/// Creates a closure which is called by webrtcbin when it added a new pad.
///
/// Pads are created for media stream negotiated using SDP.
pub(super) fn webrtcbin_on_pad_added(
    participant_id: Uuid,
    subscribe_bin: WeakRef<gst::Bin>,
    subscriber_audiomixer: WeakRef<gst::Element>,
) -> impl Fn(&[Value]) -> Option<Value> {
    move |values| {
        let subscribe_bin = subscribe_bin.upgrade()?;
        let pad = values[1]
            .get::<gst::Pad>()
            .expect("values[1] of pad-added is a gst::Pad");

        // Make sure this is a source pad
        if pad.direction() != gst::PadDirection::Src {
            log::error!("Got sink pad in subscriber webrtbin");
            return None;
        }

        if let Err(e) = try_webrtcbin_on_pad_added(
            pad,
            participant_id,
            subscribe_bin,
            subscriber_audiomixer.clone(),
        ) {
            log::error!("Failed to handle subscriber-webrtc's pad-added event, {e:?}",);
        }

        None
    }
}

fn try_webrtcbin_on_pad_added(
    pad: gst::Pad,
    participant_id: Uuid,
    subscribe_bin: gst::Bin,
    subscriber_audiomixer: WeakRef<gst::Element>,
) -> Result<()> {
    // Create a decodebin which will decode the rtp to raw media (audio/video)
    let decodebin = gst::ElementFactory::make("decodebin").build()?;

    // Handle new source pads created by decodebin
    decodebin.connect(
        "pad-added",
        true,
        decodebin_on_pad_added(
            participant_id,
            subscribe_bin.downgrade(),
            subscriber_audiomixer,
        ),
    );

    // Add the decodebin to the subscriber bin and sync its current running state
    subscribe_bin.add(&decodebin)?;
    decodebin.sync_state_with_parent()?;

    // link the new pad with the decodebin
    let decode_sink_pad = decodebin
        .static_pad("sink")
        .expect("decodebin has a static_pad named `sink`");
    pad.link(&decode_sink_pad)?;

    Ok(())
}

/// Creates a closure which is called by gstreamer when the decodebin object added a new pad.
///
/// Pads are created for every media format decodebin recogizes on its sink
pub(super) fn decodebin_on_pad_added(
    participant_id: Uuid,
    subscribe_bin: WeakRef<gst::Bin>,
    subscriber_audiomixer: WeakRef<gst::Element>,
) -> impl Fn(&[Value]) -> Option<Value> {
    move |values| {
        let subscribe_bin = subscribe_bin.upgrade()?;
        let subcriber_audiomixer = subscriber_audiomixer.upgrade()?;
        let pad = values[1]
            .get::<gst::Pad>()
            .expect("values[1] of pad-added is a gst::Pad");

        // Make sure this is a source pad
        if pad.direction() != gst::PadDirection::Src {
            log::error!("Got sink pad in subscriber webrtbin");
            return None;
        }

        if let Err(e) =
            try_decodebin_on_pad_added(pad, participant_id, subscribe_bin, subcriber_audiomixer)
        {
            log::error!("Failed to handle decodebin's pad-added event, {e:?}");
        }

        None
    }
}

fn try_decodebin_on_pad_added(
    pad: gst::Pad,
    participant_id: Uuid,
    subscribe_bin: gst::Bin,
    subcriber_audiomixer: gst::Element,
) -> Result<()> {
    // Check what kind of media this pad emits
    let caps = pad.caps().context("no caps in added pad")?;
    let caps = caps.structure(0).context("empty caps list")?;

    // If it isn't audio, we don't care connect it to a fakesink
    if !caps.name().starts_with("audio") {
        log::warn!(
            "Receiving non-audio media (got {}) in subscription to {participant_id}",
            caps.name()
        );

        let fakesink = gst::ElementFactory::make("fakesink").build()?;
        subscribe_bin.add(&fakesink)?;

        let sink = fakesink
            .static_pad("sink")
            .expect("fakesink has a static_pad named `sink`");
        pad.link(&sink)?;
        fakesink.sync_state_with_parent()?;
        return Ok(());
    }

    // Create elements to process the incoming audio
    let queue = gst::ElementFactory::make("queue").build()?;
    let audioconvert = gst::ElementFactory::make("audioconvert").build()?;
    let audioresample = gst::ElementFactory::make("audioresample").build()?;

    // Add and link them all
    subscribe_bin.add_many(&[&queue, &audioconvert, &audioresample])?;
    gst::Element::link_many(&[&queue, &audioconvert, &audioresample])?;

    // Link the queue's sink to our new pad
    let sink = queue
        .static_pad("sink")
        .expect("queue has a static_pad named `sink`");
    pad.link(&sink)?;

    // Create a ghost/proxypad to link to the audiomixer outside the subscribe bin
    let source = audioresample
        .static_pad("src")
        .expect("audioresample has a static_pad named `sink`");
    let ghost_pad = gst::GhostPad::with_target(None, &source)?;
    subscribe_bin.add_pad(&ghost_pad)?;

    let mixer_sink = subcriber_audiomixer
        .request_pad_simple("sink_%u")
        .context("Failed to request sink-pad on subscriber-audiomixer")?;
    ghost_pad.link(&mixer_sink)?;

    queue.sync_state_with_parent()?;
    audioconvert.sync_state_with_parent()?;
    audioresample.sync_state_with_parent()?;

    Ok(())
}
