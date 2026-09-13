// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{ops::Deref, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use glib::{ControlFlow, GString};
use gstreamer::{bus::BusWatchGuard, prelude::*, MessageView, Object, Pipeline};
use log::{max_level, Level};
use parking_lot::Mutex;
use tokio::{sync::oneshot, time::timeout};

use crate::debug;

type CallbackFn = dyn FnMut(&Pipeline, MessageView) + Send + Sync;

pub(crate) struct PipelineWatched {
    pipeline: Pipeline,
    eos: Option<oneshot::Receiver<()>>,
    callbacks: Arc<Mutex<Vec<Box<CallbackFn>>>>,
    _bus_watch_guard: Option<BusWatchGuard>,
}

impl std::fmt::Debug for PipelineWatched {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl PipelineWatched {
    pub(crate) fn new(name: &str, init_bus_watch: bool, wait_for_eos: bool) -> Result<Self> {
        let pipeline = Pipeline::with_name(name);

        let callbacks: Vec<Box<CallbackFn>> = Vec::new();
        let callbacks = Arc::new(Mutex::new(callbacks));

        let bus_watch_guard = if init_bus_watch {
            let bus = pipeline.bus().context("failed to get bus of pipeline")?;

            let pipeline_weak = pipeline.downgrade();
            let bus_watch_guard = bus.add_watch({
                let callbacks = callbacks.clone();
                move |_, msg| {
                    let Some(pipeline) = pipeline_weak.upgrade() else {
                        log::error!("upgrade pipeline fail bus watcher failed");
                        return ControlFlow::Continue;
                    };

                    for callback in callbacks.lock().iter_mut() {
                        callback(&pipeline, msg.view());
                    }

                    match msg.view() {
                        MessageView::Error(err) => {
                            log_message(
                                &pipeline,
                                err.src(),
                                &err.error(),
                                err.debug().as_ref(),
                                Level::Error,
                            );
                        }
                        MessageView::Warning(warn) => {
                            log_message(
                                &pipeline,
                                warn.src(),
                                &warn.error(),
                                warn.debug().as_ref(),
                                Level::Warn,
                            );
                        }
                        MessageView::Info(info) => {
                            log_message(
                                &pipeline,
                                info.src(),
                                &info.error(),
                                info.debug().as_ref(),
                                Level::Info,
                            );
                        }
                        MessageView::Latency(_) => {
                            let _ = pipeline.recalculate_latency();
                        }
                        _ => (),
                    }
                    ControlFlow::Continue
                }
            })?;

            Some(bus_watch_guard)
        } else {
            None
        };

        let mut this = Self {
            pipeline,
            eos: None,
            callbacks,
            _bus_watch_guard: bus_watch_guard,
        };

        if wait_for_eos {
            let (eos_tx, eos_rx) = oneshot::channel();

            this.add_watch({
                let mut eos_tx = Some(eos_tx);

                move |_, message_view| {
                    if let MessageView::Eos(_) = message_view {
                        match eos_tx.take() {
                            Some(eos_tx) => {
                                if eos_tx.send(()).is_err() {
                                    log::error!(
                                        "unable to send eos signal to the oneshot channel in BusWatcher"
                                    );
                                }
                            }
                            None => {
                                log::debug!("oneshot channel already received an eos signal, skip");
                            }
                        }
                    }
                }
            });

            this.eos = Some(eos_rx);
        }

        Ok(this)
    }

    pub(crate) fn add_watch<F>(&self, callback: F)
    where
        F: FnMut(&Pipeline, MessageView) + Send + Sync + 'static,
    {
        self.callbacks.lock().push(Box::new(callback));
    }

    pub(crate) async fn close(&mut self) {
        log::debug!("close pipeline {}", self.pipeline.name());

        debug::debug_dot(&self.pipeline, &format!("drop-{}", self.pipeline.name()));

        if let Some(eos) = self.eos.take() {
            self.pipeline.send_event(gstreamer::event::Eos::new());

            trace!("wait for eos");

            match timeout(Duration::from_secs(8), eos).await {
                Ok(Ok(())) => {}
                Ok(Err(_e)) => {
                    log::error!(
                        "Failed to receive EOS signal for pipeline {}",
                        self.pipeline.name()
                    );
                }
                Err(_) => {
                    log::error!(
                        "Timeout while waiting for EOS signal for pipeline {}",
                        self.pipeline.name()
                    );
                }
            }
        }

        if let Err(error) = self.pipeline.set_state(gstreamer::State::Null) {
            log::error!("Unable to set the pipeline to the `Null` state, error: {error}");
        }

        log::debug!("close for pipeline {} is done", self.pipeline.name());
    }
}

impl Drop for PipelineWatched {
    fn drop(&mut self) {
        if self.pipeline.current_state() == gstreamer::State::Null {
            return;
        }

        log::warn!(
            "Pipeline {} was dropped without being closed, setting it to the Null",
            self.pipeline.name()
        );

        if let Err(err) = self.pipeline.set_state(gstreamer::State::Null) {
            log::error!("Failed to set the pipeline to Null state, {err}");
        }
    }
}

impl Deref for PipelineWatched {
    type Target = Pipeline;

    fn deref(&self) -> &Self::Target {
        &self.pipeline
    }
}

fn log_message(
    pipeline: &Pipeline,
    src: Option<&Object>,
    error: &glib::Error,
    debug: Option<&GString>,
    level: Level,
) {
    log::log!(
        level,
        "Received bus message for element {:?}: {error}",
        src.map(GstObjectExt::path_string),
    );

    if let Some(info) = debug {
        log::debug!("Debugging information: {info}");
    }

    if max_level() >= level {
        debug::debug_dot(pipeline, &format!("BUS-{level}"));
    }
}
