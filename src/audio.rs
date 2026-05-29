use std::{
    fs::File,
    io::BufReader,
    time::{Duration, Instant},
};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

pub struct AudioPlayer {
    pub(crate) _stream: OutputStream,
    pub(crate) handle: OutputStreamHandle,
    pub(crate) current_sink: Option<Sink>,
    pub(crate) volume: f32,
    pub(crate) started_at: Option<Instant>,
    pub(crate) accumulated: Duration,
    pub(crate) track_total: Duration,
    pub(crate) finished_handled: bool,
}

impl AudioPlayer {
    pub fn new() -> anyhow::Result<Self> {
        let (stream, handle) = OutputStream::try_default()?;
        Ok(Self {
            _stream: stream,
            handle,
            current_sink: None,
            volume: 1.0,
            started_at: None,
            accumulated: Duration::ZERO,
            track_total: Duration::ZERO,
            finished_handled: true,
        })
    }

    pub fn play_file(&mut self, file_path: &str, total_seconds: i64) -> anyhow::Result<()> {
        if let Some(sink) = self.current_sink.take() {
            sink.stop();
        }
        let file = File::open(file_path)?;
        let source = Decoder::new(BufReader::new(file))?;
        let sink = Sink::try_new(&self.handle)?;
        sink.set_volume(self.volume);
        sink.append(source);
        sink.play();
        self.current_sink = Some(sink);
        self.started_at = Some(Instant::now());
        self.accumulated = Duration::ZERO;
        self.track_total = Duration::from_secs(total_seconds.max(0) as u64);
        self.finished_handled = false;
        Ok(())
    }

    pub fn toggle_pause(&mut self) -> Option<bool> {
        let paused = self.current_sink.as_ref()?.is_paused();
        if paused {
            self.started_at = Some(Instant::now());
            self.current_sink.as_ref()?.play();
            Some(false)
        } else {
            if let Some(started) = self.started_at.take() {
                self.accumulated += started.elapsed();
            }
            self.current_sink.as_ref()?.pause();
            Some(true)
        }
    }

    pub fn is_paused(&self) -> bool {
        self.current_sink.as_ref().is_some_and(|s| s.is_paused())
    }

    /// Best-effort playback position via wall-clock timer.
    pub fn position(&self) -> Duration {
        let running = self.started_at.map(|s| s.elapsed()).unwrap_or(Duration::ZERO);
        let pos = self.accumulated + running;
        if self.track_total > Duration::ZERO {
            pos.min(self.track_total)
        } else {
            pos
        }
    }

    pub fn total(&self) -> Duration {
        self.track_total
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if let Some(sink) = &self.current_sink {
            sink.set_volume(self.volume);
        }
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn has_track(&self) -> bool {
        self.current_sink.is_some()
    }

    /// Returns true exactly once when the current track finishes.
    pub fn take_finished(&mut self) -> bool {
        if self.finished_handled {
            return false;
        }
        match &self.current_sink {
            Some(sink) if sink.empty() => {
                self.finished_handled = true;
                true
            }
            _ => false,
        }
    }
}
