use rodio::source::{SineWave, Source};
use rodio::{OutputStream, Sink};
use std::time::Duration;

/// Play a short "recording started" cue (rising 800 → 1200 Hz).
pub fn play_start() {
    std::thread::spawn(|| {
        let _ = play_two_tones(800.0, 1200.0, 45);
    });
}

/// Play a short "recording stopped" cue (falling 1200 → 800 Hz).
pub fn play_stop() {
    std::thread::spawn(|| {
        let _ = play_two_tones(1200.0, 800.0, 45);
    });
}

fn play_two_tones(first_hz: f32, second_hz: f32, each_ms: u64) -> Result<(), ()> {
    let (_stream, handle) = OutputStream::try_default().map_err(|_| ())?;
    let sink = Sink::try_new(&handle).map_err(|_| ())?;
    let a = SineWave::new(first_hz)
        .take_duration(Duration::from_millis(each_ms))
        .amplify(0.12)
        .fade_in(Duration::from_millis(8));
    let b = SineWave::new(second_hz)
        .take_duration(Duration::from_millis(each_ms))
        .amplify(0.12);
    sink.append(a);
    sink.append(b);
    sink.sleep_until_end();
    Ok(())
}
