use crate::audio::capture::AudioCapture;
use crate::audio::ringbuffer::RingBuffer;
use crate::error::{AppError, Result};
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::oneshot;

enum Command {
    Start {
        device: Option<String>,
        reply: oneshot::Sender<Result<()>>,
    },
    Stop {
        reply: oneshot::Sender<Vec<f32>>,
    },
}

/// Thread-safe handle to an `AudioCapture` that runs on a dedicated OS thread.
///
/// `cpal::Stream` is `!Send` (on Linux, `PhantomData<*mut ()>` inside it
/// prevents crossing thread boundaries via normal Rust moves). We work around
/// this by constructing `AudioCapture` *inside* the thread, and pre-sharing
/// only the `Arc<Mutex<…>>` fields (which are `Send`) before spawning.
///
/// The controller itself is `Send + Sync` and safe to hold inside an async
/// `Orchestrator`.
pub struct AudioController {
    cmd_tx: std::sync::mpsc::SyncSender<Command>,
    /// Shared level meter — updated by the audio callback in real time.
    pub level: Arc<Mutex<f32>>,
    /// Read-only reference to the ring buffer (for diagnostics / UI).
    pub buffer_ref: Arc<Mutex<RingBuffer>>,
}

impl AudioController {
    /// Spawn the audio thread and return a controller handle.
    ///
    /// `max_seconds` caps the ring buffer length (e.g. 120 s).
    pub fn spawn(max_seconds: u32) -> Self {
        // Build the shared Arc fields here so they're available to callers
        // before the first Start command arrives.  The thread receives
        // clones of these Arcs, constructs AudioCapture with them, then
        // starts its command loop.  We never move an AudioCapture (or the
        // !Send Stream it will hold) across thread boundaries.
        let level: Arc<Mutex<f32>> = Arc::new(Mutex::new(0.0));
        let buffer_ref: Arc<Mutex<RingBuffer>> =
            Arc::new(Mutex::new(RingBuffer::with_seconds(max_seconds, crate::audio::capture::TARGET_SAMPLE_RATE)));

        let level_thread = level.clone();
        let buffer_thread = buffer_ref.clone();

        // Bounded channel — backpressure prevents flooding the audio thread.
        let (cmd_tx, cmd_rx) = std::sync::mpsc::sync_channel::<Command>(4);

        std::thread::Builder::new()
            .name("dss-audio".into())
            .spawn(move || {
                // AudioCapture is constructed *inside* the thread, so the
                // !Send Stream it will eventually hold never crosses a thread
                // boundary.
                let mut capture = AudioCapture::with_shared(level_thread, buffer_thread);
                while let Ok(cmd) = cmd_rx.recv() {
                    match cmd {
                        Command::Start { device, reply } => {
                            let r = capture.start(device.as_deref());
                            let _ = reply.send(r);
                        }
                        Command::Stop { reply } => {
                            capture.stop();
                            let samples = capture.buffer.lock().drain_to_vec();
                            let _ = reply.send(samples);
                        }
                    }
                }
                // Channel closed — audio thread exits cleanly.
            })
            .expect("failed to spawn audio thread");

        Self { cmd_tx, level, buffer_ref }
    }

    /// Start recording on the given device (or the default device if `None`).
    pub async fn start_recording(&self, device: Option<String>) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::Start { device, reply: tx })
            .map_err(|e| AppError::Audio(e.to_string()))?;
        rx.await
            .map_err(|e| AppError::Audio(e.to_string()))?
    }

    /// Stop recording and return all buffered PCM samples (16 kHz mono f32).
    pub async fn stop_and_drain(&self) -> Result<Vec<f32>> {
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send(Command::Stop { reply: tx })
            .map_err(|e| AppError::Audio(e.to_string()))?;
        rx.await
            .map_err(|e| AppError::Audio(e.to_string()))
    }

    /// Snapshot of the current RMS level (0.0 – 1.0) for UI meters.
    pub fn level_snapshot(&self) -> f32 {
        *self.level.lock()
    }
}
