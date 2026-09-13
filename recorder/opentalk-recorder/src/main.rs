// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![allow(clippy::module_name_repetitions)]

use std::{
    collections::HashMap,
    net::IpAddr,
    process::{exit, Command, Stdio},
    sync::Arc,
};

use anyhow::{Context, Result};
use async_trait::async_trait;
use axum::Router;
use clap::Parser;
use gstreamer::glib;
use log::warn;
use opentalk_client_signaling::{
    config::ClientConfig, opentalk_types::api::v1::error::ApiError, AuthConfig, OpenTalkClient,
};
use opentalk_orchestrator_client::{
    client::OrchestratorHandle, OrchestratorClient, ServiceAddress,
};
use opentalk_recorder_web_api::v1::{self, RecorderBackend, RecordingAction};
use opentalk_service_auth::service::ApiKeyAuthorization;
use opentalk_types_api_internal::recording::RecordingTarget;
use service_probe::{set_service_state, start_probe, ServiceState};
use service_probe_client::is_ready;
use settings::{HardwareAcceleration, HardwareAccelerationIntel, MonitoringSettings, Settings};
use system_info::{cpu::cpu_usage_poll, gpu_intel::gpu_intel_usage_poll};
use tokio::{
    net::TcpListener,
    select,
    signal::{
        ctrl_c,
        unix::{signal, SignalKind},
    },
    sync::{broadcast, Mutex},
};

use crate::{
    cli::{print_info, Args, Commands},
    orchestrator_metrics::OrchestratorStateProvider,
    recorder::{Recorder, RecordingSessionHandle},
    system_info::is_new_recording_feasible,
};

mod cli;
mod orchestrator_metrics;
mod recorder;
mod settings;
mod system_info;

type RecorderTasks = Arc<Mutex<HashMap<RecordingTarget, RecordingSessionHandle>>>;

#[derive(Clone)]
pub struct AppState {
    pub(crate) recorder_context: Arc<Recorder>,
    pub(crate) orchestrator_handle: Option<OrchestratorHandle>,
    pub(crate) tasks: RecorderTasks,
}

#[async_trait]
impl RecorderBackend for AppState {
    async fn init(&self, recording_target: RecordingTarget) -> Result<RecordingAction, ApiError> {
        if !is_new_recording_feasible() {
            return Err(ApiError::service_unavailable()
                .with_code("out_of_resources")
                .with_message("No compute resources available"));
        }

        let recorder_context = self.recorder_context.clone();

        let mut tasks = self.tasks.lock().await;

        if tasks
            .get(&recording_target)
            .is_some_and(|handle| !handle.is_finished())
        {
            return Ok(RecordingAction::AlreadyRunning);
        }

        let session = recorder_context
            .spawn_session(recording_target, self.orchestrator_handle.clone())
            .await;

        match session {
            Ok(task) => {
                tasks.insert(recording_target, task);
                Ok(RecordingAction::Created)
            }
            Err(err) => {
                log::error!("Failed to start recording session: {err:?}");

                Err(ApiError::internal().with_message("Failed to start recording session"))
            }
        }
    }
}

const DOT_OUTPUT_PATH: &str = "./pipelines";

fn check_plugins() -> Result<()> {
    let registry = gstreamer::Registry::get();

    let required = [
        "audiomixer",
        "audiotestsrc",
        "autodetect",
        "compositor",
        "debug",
        "dtls",
        "pango",
        "png",
        "rtp",
        "srtp",
        "udp",
        "videotestsrc",
        "vpx",
        "webrtc",
    ];

    #[cfg(target_os = "linux")]
    let required = {
        let mut v = required.to_vec();
        v.push("vaapi");
        v
    };

    let failed_plugins: Vec<_> = required
        .into_iter()
        .filter(|plug| registry.find_plugin(plug).is_none())
        .collect();

    if !failed_plugins.is_empty() {
        anyhow::bail!(
            "Failed to load GStreamer plugins [{}], try to start the application with 'GST_DEBUG=1' if the plugins are installed correctly.",
            failed_plugins.join(", ")
        );
    }

    Ok(())
}

fn check_intel_gpu_top_command() -> Result<()> {
    let status = Command::new("intel_gpu_top")
        .arg("-h")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .map(|status| status.success());

    if status.is_err() || status.ok() == Some(false) {
        anyhow::bail!(
            "The intel_gpu_top command is not installed, this is mandatory for hardware acceleration, please install it or remove hardware acceleration."
        );
    }

    Ok(())
}

/// `rustls` depend on a `CryptoProvider` being configured.
/// If no provider was explicitly configured, a provider will be derived from
/// the enabled features. Since there are many crates that depend on `rustls`, we
/// don't have complete control over the enabled features.
/// If the configuration via feature is ambiguous `rustls` will panic.
///
/// Here we ensure that these crates are explicitly configured.
fn ensure_crypto_provider() {
    rustls::crypto::CryptoProvider::install_default(rustls::crypto::aws_lc_rs::default_provider())
        .expect("valid default crypto provider expected");

    jsonwebtoken::crypto::CryptoProvider::install_default(
        &jsonwebtoken::crypto::aws_lc::DEFAULT_PROVIDER,
    )
    .expect("valid default crypto provider expected");
}

#[cfg(not(target_os = "macos"))]
#[tokio::main]
async fn main() -> Result<()> {
    ensure_crypto_provider();
    main_loop().await.expect("failed to run main loop");
    Ok(())
}

#[cfg(target_os = "macos")]
fn main() {
    use std::sync::mpsc::channel;

    use objc2::{
        define_class, msg_send, rc::Retained, runtime::ProtocolObject, DefinedClass,
        MainThreadMarker, MainThreadOnly,
    };
    use objc2_app_kit::{
        NSApplication, NSApplicationDelegate, NSEvent, NSEventModifierFlags, NSEventSubtype,
        NSEventType,
    };
    use objc2_foundation::{NSNotification, NSObject, NSObjectProtocol, NSPoint};

    struct AppDelegateIvars {
        send: std::sync::mpsc::Sender<()>,
    }

    define_class!(
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "AppDelegate"]
        #[ivars = AppDelegateIvars]
        struct AppDelegate;

        unsafe impl NSObjectProtocol for AppDelegate {}

        unsafe impl NSApplicationDelegate for AppDelegate {
            #[unsafe(method(applicationDidFinishLaunching:))]
            fn did_finish_launching(&self, _notification: &NSNotification) {
                self.ivars()
                    .send
                    .send(())
                    .expect("failed to send to main thread");
            }
        }
    );

    impl AppDelegate {
        fn new(send: std::sync::mpsc::Sender<()>, mtm: MainThreadMarker) -> Retained<Self> {
            let this = Self::alloc(mtm);
            let this = this.set_ivars(AppDelegateIvars { send });
            unsafe { msg_send![super(this), init] }
        }
    }

    ensure_crypto_provider();

    let mtm = MainThreadMarker::new().expect("must be called on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    let (send, recv) = channel::<()>();

    let delegate = AppDelegate::new(send, mtm);
    let object = ProtocolObject::from_ref(&*delegate);
    app.setDelegate(Some(object));

    let t = std::thread::spawn(move || {
        // Wait for the NSApp to launch to avoid possibly calling stop_() too early
        recv.recv().expect("failed to receive from main thread");

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");
        let res = runtime.block_on(main_loop());

        // Must use MainThreadMarker::new_unchecked since we're not on the main thread,
        // but NSApplication methods are safe to call from any thread for stop/postEvent.
        let mtm = unsafe { MainThreadMarker::new_unchecked() };
        let app = NSApplication::sharedApplication(mtm);
        app.stop(None);

        // Stopping the event loop requires posting an actual event
        let event = NSEvent::otherEventWithType_location_modifierFlags_timestamp_windowNumber_context_subtype_data1_data2(
            NSEventType::ApplicationDefined,
            NSPoint { x: 0.0, y: 0.0 },
            NSEventModifierFlags::empty(),
            0.0,
            0,
            None,
            NSEventSubtype::ApplicationActivated.0,
            0,
            0,
        );
        if let Some(event) = event {
            app.postEvent_atStart(&event, true);
        }

        res
    });

    app.run();

    let res = t.join().expect("failed to join thread");
    if let Err(e) = res {
        eprintln!("Exit on failure: {e:?}");
        std::process::exit(-1);
    }
}

async fn main_loop() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    if args.info.should_print() {
        print_info(&args.info);
        return Ok(());
    }

    let settings = Arc::new(settings::Settings::load(args.config.as_ref())?);
    if let Some(Commands::Health { endpoint }) = args.command {
        let Some(monitoring_endpoint) = endpoint.or_else(|| {
            settings.monitoring.as_ref().map(|monitoring_settings| {
                format!(
                    "http://{}:{}",
                    monitoring_settings.addr, monitoring_settings.port
                )
                .parse()
                .expect("valid endpoint can be built from monitoring settings")
            })
        }) else {
            log::warn!("Monitoring not configured and no url endpoint parameter given");
            exit(1);
        };
        return match is_ready(&monitoring_endpoint).await {
            Ok(true) => {
                log::info!("READY");
                Ok(())
            }
            Ok(false) => {
                log::info!("Not Ready");
                exit(1)
            }
            Err(err) => {
                log::error!("Err: {err}");
                exit(-1)
            }
        };
    }
    if std::env::var("GST_DEBUG_DUMP_DOT_DIR").is_err() {
        warn!(
            "Using default dot path. You need to set GST_DEBUG_DUMP_DOT_DIR in environment to an absolute path to get DOT output."
        );
        unsafe {
            std::env::set_var("GST_DEBUG_DUMP_DOT_DIR", DOT_OUTPUT_PATH);
        }
    }

    tokio::task::spawn_blocking(move || -> Result<()> {
            if std::env::var("GST_DEBUG_DUMP_DOT_DIR").is_err() {
                warn!(
                    "Using default dot path. You need to set GST_DEBUG_DUMP_DOT_DIR in environment to an absolute path to get DOT output."
                );
                unsafe {
                    std::env::set_var("GST_DEBUG_DUMP_DOT_DIR", DOT_OUTPUT_PATH);
                }
            }

            gstreamer::init()?;
            check_plugins()?;

            // Run a MainLoop on a separate thread so gstreamer bus watches work
            let gstreamer_main_loop = glib::MainLoop::new(None, false);
            std::thread::spawn({
                let main_loop = gstreamer_main_loop.clone();

                move || {
                    main_loop.run();
                }
            });

            let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .context("failed to start tokio async runtime")?;


            runtime.spawn(async move {
                let mut sig_term =
                    signal(SignalKind::terminate()).expect("can not setup SIGTERM handler");
                select! {
                    _ = ctrl_c() => { log::info!("received Ctrl-C"); }
                    _ = sig_term.recv() => { log::info!("received SIGTERM"); }
                }
                shutdown_tx
                    .send(())
                    .expect("failed to send shutdown signal");
            });


            if let Err(e) = runtime.block_on(run_recorder(shutdown_rx, settings)) {
                eprintln!("Exit on failure: {e:?}");
                std::process::exit(-1);
            }

            log::debug!("Send quit to main_loop");
            gstreamer_main_loop.quit();
            Ok(())
        }).await??;
    Ok(())
}

async fn run_recorder(
    mut shutdown_rx: broadcast::Receiver<()>,
    settings: Arc<Settings>,
) -> Result<()> {
    let client = OpenTalkClient::create(ClientConfig {
        auth: AuthConfig::ApiKey(settings.controller.api_key.clone()),
        controller: settings.controller.url.clone(),
    })
    .await?;

    let recorder_context = Recorder::new(settings.clone(), client, shutdown_rx.resubscribe());

    let recorder_tasks = Arc::new(Mutex::new(HashMap::new()));
    let mut orchestrator_handle = None;

    if let Some(config) = settings.orchestrator.clone() {
        let recorder_key_ids = settings
            .http
            .api_keys
            .inner()
            .iter()
            .map(|api_key| api_key.id.clone());

        let (client, handle) = OrchestratorClient::create(config, recorder_key_ids).await;
        orchestrator_handle = Some(handle);

        let state_provider = OrchestratorStateProvider {
            tasks: Arc::clone(&recorder_tasks),
        };

        let port = settings.http.port;

        let mut shutdown_rx_tmp = shutdown_rx.resubscribe();
        let wait_shutdown = async move {
            let _ = shutdown_rx_tmp.recv().await;
        };

        tokio::task::spawn(async move {
            client
                .run(ServiceAddress::Port(port), state_provider, wait_shutdown)
                .await
        });
    }

    let recorder = AppState {
        tasks: recorder_tasks,
        orchestrator_handle,
        recorder_context: Arc::new(recorder_context.clone()),
    };

    if let Some(MonitoringSettings { port, addr }) = settings.monitoring {
        start_probe(addr, port, ServiceState::Up).await?;
    }

    let auth_middleware = settings
        .http
        .api_keys
        .auth_middleware()
        .context("Invalid API key configuration")?;

    select! {
        _ =  shutdown_rx.recv() => {
            log::info!("Received shutdown, shutdown all remaining tasks");
        }
        _ = run_usage_polling(&recorder_context) => {
            log::debug!("Usage polling failed, shutdown all remaining tasks");
        }
        result = run_axum_server(settings.http.addr, settings.http.port, recorder.clone(), auth_middleware) => { result?; }
    }

    log::info!("All tasks are finished");

    Ok(())
}

async fn run_axum_server(
    address: IpAddr,
    port: u16,
    recorder: AppState,
    auth_middleware: ApiKeyAuthorization,
) -> Result<()> {
    let app = Router::new()
        .merge(v1::routes().layer(auth_middleware))
        .with_state(recorder);

    let listener = TcpListener::bind((address, port)).await?;

    // Server up and running, ready to process requests
    set_service_state(ServiceState::Ready);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn run_usage_polling(recorder_context: &Recorder) -> Result<(), broadcast::error::RecvError> {
    let hardware_acceleration = recorder_context
        .settings
        .recorder
        .clone()
        .and_then(|s| s.hardware_acceleration);
    let mut cutoff = settings::default_max_load();
    if let Some(recorder_info) = &recorder_context.settings.recorder {
        cutoff = recorder_info.max_load;
    }

    let run_blocking = move || -> Result<()> {
        if let Some(HardwareAcceleration::Intel(HardwareAccelerationIntel { device })) =
            hardware_acceleration
        {
            log::info!("Hardware Acceleration enabled, using the GPU for encoding");
            check_intel_gpu_top_command()?;
            gpu_intel_usage_poll(cutoff, device)?;
        } else {
            log::info!("Hardware Acceleration disabled, this can cause high cpu load");
            cpu_usage_poll(cutoff)?;
        }

        Ok(())
    };

    let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);
    std::thread::spawn(move || {
        if let Err(err) = run_blocking() {
            log::error!("Usage polling failed, received: {err}");
            shutdown_tx
                .send(())
                .expect("Unable to send shutdown based on usage polling error");
        }
    });

    shutdown_rx.recv().await
}
