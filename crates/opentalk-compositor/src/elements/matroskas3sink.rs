// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Context;
use glib::types::StaticType;

glib::wrapper! {
    pub struct MatroskaS3Sink(ObjectSubclass<imp::MatroskaS3Sink>)
        @extends gstreamer::Object, gstreamer::Element, gstreamer_base::BaseSink;
}

// This should be 5 MiB, regarding the S3 specifications
pub(crate) const DEFAULT_CHUNK_SIZE: u64 = 5 * 1024 * 1024;

/// # Errors
///
/// Returns an error if Gstreamer is not initialized or this function was already called in this proccess.
pub fn register() -> anyhow::Result<()> {
    gstreamer::Element::register(
        None,
        "opentalk-matroska-s3-sink",
        gstreamer::Rank::NONE,
        MatroskaS3Sink::static_type(),
    )
    .context("Failed to register opentalk-matroska-s3-sink")
}

mod imp {
    use std::{
        io::{self, Cursor, Write},
        mem::take,
        sync::LazyLock,
    };

    use anyhow::{Context, Result};
    use glib::{object::ObjectExt, value::ToValue, BorrowedObject, ParamSpecBuilderExt};
    use gstreamer::{
        format::Bytes,
        glib::{self, subclass::Signal},
        prelude::StaticType,
        subclass::prelude::{
            ElementImpl, GstObjectImpl, ObjectImpl, ObjectSubclass, ObjectSubclassExt,
        },
        EventView, Format, GenericFormattedValue, QueryViewMut, StreamError,
    };
    use gstreamer_base::subclass::prelude::{BaseSinkImpl, BaseSinkImplExt};
    use parking_lot::Mutex;

    use super::DEFAULT_CHUNK_SIZE;

    static CAT: LazyLock<gstreamer::DebugCategory> = LazyLock::new(|| {
        gstreamer::DebugCategory::new(
            "opentalk-matroska-s3-sink",
            gstreamer::DebugColorFlags::empty(),
            Some("OpenTalk Matroska Chunks"),
        )
    });

    #[derive(Default)]
    pub struct MatroskaS3Sink {
        inner: Mutex<Inner>,
    }

    struct Inner {
        // Properties
        chunk_size: u64,

        // State
        matroska_header: Vec<u8>,

        current_buffer: Cursor<Vec<u8>>,
        total_bytes_produced: u64,

        first_s3_chunk: Vec<u8>,
        // is extended until larger than 5MiB or EOS is received
        current_s3_chunk: Vec<u8>,

        total_s3_chunks_produced: u64,
    }

    impl Default for Inner {
        fn default() -> Self {
            Self {
                chunk_size: DEFAULT_CHUNK_SIZE,
                matroska_header: vec![],
                current_buffer: Cursor::new(vec![]),
                total_bytes_produced: 0,
                first_s3_chunk: vec![],
                current_s3_chunk: vec![],
                total_s3_chunks_produced: 0,
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MatroskaS3Sink {
        const NAME: &'static str = "OpenTalkMatroskaUploadSink";
        type Type = super::MatroskaS3Sink;
        type ParentType = gstreamer_base::BaseSink;
    }

    impl ObjectImpl for MatroskaS3Sink {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: LazyLock<Vec<Signal>> = LazyLock::new(|| {
                vec![Signal::builder("part")
                    .param_types([u64::static_type(), glib::Bytes::static_type()])
                    .build()]
            });

            &SIGNALS
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: LazyLock<Vec<glib::ParamSpec>> = LazyLock::new(|| {
                vec![glib::ParamSpecUInt64::builder("chunk-size")
                    .readwrite()
                    .blurb("chunk size in bytes")
                    .default_value(DEFAULT_CHUNK_SIZE)
                    .build()]
            });

            &PROPERTIES
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "chunk-size" => match value.get::<u64>() {
                    Ok(value) => self.inner.lock().chunk_size = value,
                    Err(e) => {
                        gstreamer::error!(
                            CAT,
                            imp = self,
                            "Failed to set paramter 'chunk-size', {e}"
                        );
                    }
                },
                name => gstreamer::error!(CAT, imp = self, "Unknown property '{name}'"),
            }
        }

        fn property(&self, _id: usize, _pspec: &glib::ParamSpec) -> glib::Value {
            self.inner.lock().chunk_size.to_value()
        }
    }

    impl GstObjectImpl for MatroskaS3Sink {}

    impl ElementImpl for MatroskaS3Sink {
        fn metadata() -> Option<&'static gstreamer::subclass::ElementMetadata> {
            static ELEMENT_METADATA: LazyLock<gstreamer::subclass::ElementMetadata> = LazyLock::new(
                || {
                    gstreamer::subclass::ElementMetadata::new(
                    "OpenTalkMatroskaS3Sink",
                    "Sink/Network",
                    "Split a matroska stream into chunks that can be uploaded using the S3 multipart API",
                    "Konstantin Baltruschat"
                )
                },
            );

            Some(&*ELEMENT_METADATA)
        }

        fn pad_templates() -> &'static [gstreamer::PadTemplate] {
            static PAD_TEMPLATES: LazyLock<Vec<gstreamer::PadTemplate>> = LazyLock::new(|| {
                let sink_pad_template = gstreamer::PadTemplate::new(
                    "sink",
                    gstreamer::PadDirection::Sink,
                    gstreamer::PadPresence::Always,
                    &gstreamer::Caps::new_any(),
                )
                .expect("unable to create PadTemplate sink for matroskas3sink");

                vec![sink_pad_template]
            });

            &PAD_TEMPLATES
        }
    }

    impl BaseSinkImpl for MatroskaS3Sink {
        fn event(&self, event: gstreamer::Event) -> bool {
            match event.view() {
                EventView::Segment(segment) => {
                    let segment = segment.segment();

                    self.inner.lock().handle_segment(segment);
                }
                EventView::Eos(_) => self.inner.lock().finish(&self.obj()),
                _ => (),
            }

            self.parent_event(event)
        }

        fn query(&self, query: &mut gstreamer::QueryRef) -> bool {
            if let QueryViewMut::Seeking(seeking) = query.view_mut() {
                seeking.set(
                    seeking.format() == Format::Bytes,
                    Bytes::from_u64(0),
                    GenericFormattedValue::Bytes(None),
                );
                true
            } else {
                self.parent_query(query)
            }
        }

        fn render(
            &self,
            buffer: &gstreamer::Buffer,
        ) -> Result<gstreamer::FlowSuccess, gstreamer::FlowError> {
            if let Err(e) = self.inner.lock().render_buffer(&self.obj(), buffer) {
                gstreamer::element_error!(
                    self.obj(),
                    StreamError::Failed,
                    ("Failed to render_buffer: {e:?}")
                );

                Err(gstreamer::FlowError::Error)
            } else {
                Ok(gstreamer::FlowSuccess::Ok)
            }
        }
    }

    impl Inner {
        fn render_buffer(
            &mut self,
            obj: &BorrowedObject<super::MatroskaS3Sink>,
            buffer: &gstreamer::Buffer,
        ) -> Result<gstreamer::FlowSuccess> {
            let readable = buffer.map_readable().context("Failed to map buffer")?;

            self.write(obj, &readable)?;

            Ok(gstreamer::FlowSuccess::Ok)
        }

        fn handle_segment(
            &mut self,
            segment: &gstreamer::FormattedSegment<GenericFormattedValue>,
        ) -> bool {
            let GenericFormattedValue::Bytes(Some(segment_start)) = segment.start() else {
                return false;
            };

            self.seek(*segment_start);

            true
        }

        fn write(
            &mut self,
            obj: &BorrowedObject<super::MatroskaS3Sink>,
            buf: &[u8],
        ) -> io::Result<usize> {
            // Skip searching large buffers to optimize performance.
            // The condition below ensures that we only search for the Matroska cluster's magic bytes in small buffers (less than 400 bytes).
            // This is based on the observation that the header chunks containing these magic bytes are typically around 100 bytes.
            // By limiting the search to smaller buffers, we avoid the expensive operation of scanning large data chunks,
            // which can significantly slow down the process. This threshold of 400 bytes provides a good balance,
            // allowing us to catch the header while minimizing unnecessary computation.
            if buf.len() < 400 {
                // Check if a new matroska cluster is written by identifying these magic bytes
                const MATROSKA_MAGIC_BYTES: [u8; 4] = [0x1F, 0x43, 0xB6, 0x75];
                let buf_contains_new_cluster: bool =
                    buf.windows(4).any(|w| w == MATROSKA_MAGIC_BYTES);

                if buf_contains_new_cluster {
                    self.total_bytes_produced += self.current_buffer.get_ref().len() as u64;

                    if self.matroska_header.is_empty() {
                        self.matroska_header
                            .clone_from(self.current_buffer.get_ref());
                    }

                    self.current_s3_chunk
                        .extend_from_slice(self.current_buffer.get_ref());
                    self.current_buffer.set_position(0);
                    self.current_buffer.get_mut().clear();

                    self.maybe_submit_s3(obj);
                }
            }

            self.current_buffer.write(buf)
        }

        fn seek(&mut self, pos: u64) {
            if let Some(new_pos) = pos.checked_sub(self.total_bytes_produced) {
                self.current_buffer.set_position(new_pos);
            } else {
                self.current_s3_chunk
                    .extend_from_slice(self.current_buffer.get_ref());

                self.current_buffer = Cursor::new(self.matroska_header.clone());
                self.current_buffer.set_position(pos);
                self.total_bytes_produced = 0;
            }
        }

        fn maybe_submit_s3(&mut self, obj: &BorrowedObject<super::MatroskaS3Sink>) {
            if (self.current_s3_chunk.len() as u64) < self.chunk_size {
                return;
            }

            let bytes = glib::Bytes::from(&self.current_s3_chunk[..]);
            obj.emit_by_name::<()>("part", &[&self.total_s3_chunks_produced, &bytes]);

            self.total_s3_chunks_produced += 1;

            if self.first_s3_chunk.is_empty() {
                self.first_s3_chunk = take(&mut self.current_s3_chunk);
            } else {
                self.current_s3_chunk.clear();
            }
        }

        fn finish(&mut self, obj: &BorrowedObject<super::MatroskaS3Sink>) {
            if self.first_s3_chunk.is_empty() {
                // No chunks were emitted written yet, so use the current one
                self.first_s3_chunk = take(&mut self.current_s3_chunk);
            }

            if !self.current_s3_chunk.is_empty() {
                // There is a pending chunk, emit it
                let bytes = glib::Bytes::from(&self.current_s3_chunk[..]);
                obj.emit_by_name::<()>("part", &[&self.total_s3_chunks_produced, &bytes]);
            }

            // Update the first chunk we've ever emitted with the current buffer (which should contain the matroska header)
            Cursor::new(&mut self.first_s3_chunk)
                .write_all(self.current_buffer.get_ref())
                .expect("unable to write all current buffer into cursor for matroskas3sink");

            let bytes = glib::Bytes::from(&self.first_s3_chunk[..]);
            obj.emit_by_name::<()>("part", &[&0u64, &bytes]);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, sync::Arc};

    use parking_lot::Mutex;

    use super::*;

    #[test]
    fn multipart_upload_simulator() {
        use gstreamer::prelude::*;

        env_logger::init();
        gstreamer::init().unwrap();
        register().unwrap();

        let main_loop = glib::MainLoop::new(None, false);

        let ml = main_loop.clone();
        std::thread::spawn(move || ml.run());

        let pipeline = gstreamer::parse::launch(
            "
        audiotestsrc
                volume=0.1
                samplesperbuffer=480
                num-buffers=1000
            ! audio/x-raw,layout=interleaved,rate=48000
            ! opusenc
            ! mux.

        videotestsrc
                num-buffers=240
            ! video/x-raw,width=1920,height=1080,framerate=24/1
            ! vp8enc
            ! mux.

        webmmux
                name=mux
                writing-app=OpenTalk
                offset-to-zero=true
            ! opentalk-matroska-s3-sink name=sink  chunk-size=5000000 sync=false",
        )
        .unwrap();
        let pipeline = pipeline.downcast::<gstreamer::Pipeline>().unwrap();

        let multipart_upload_simulator = <Arc<Mutex<BTreeMap<u64, Vec<u8>>>>>::default();

        let sink = pipeline.by_name("sink").unwrap();
        let multipart_upload_simulator_ = multipart_upload_simulator.clone();
        sink.connect("part", false, move |v: &[glib::Value]| {
            let n = v[1].get::<u64>().unwrap();
            let b = v[2].get::<glib::Bytes>().unwrap();

            multipart_upload_simulator_.lock().insert(n, b.to_vec());

            None
        });

        pipeline.set_state(gstreamer::State::Playing).unwrap();

        let bus = pipeline.bus().unwrap();

        std::thread::sleep(std::time::Duration::from_secs(12));

        pipeline.send_event(gstreamer::event::Eos::new());

        for e in bus.iter_timed(None) {
            if let gstreamer::MessageView::Eos(_) = e.view() {
                break;
            }
        }

        pipeline.set_state(gstreamer::State::Null).unwrap();

        main_loop.quit();

        let mut file = vec![];

        for chunk in multipart_upload_simulator.lock().values() {
            file.extend_from_slice(chunk);
        }

        // expect at least 1MiB of data
        assert!(file.len() > 1_000_000, "suspicious file size");

        let _ = std::fs::write("test_output/multipart_upload_simulator.mkv", &file);
    }
}
