#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppState {
    Idle,
    Recording,
    Transcribing,
    Injecting,
    Error,
}

#[derive(Debug)]
pub enum Transition {
    StartRecording,
    StopRecording,
    TranscriptionDone,
    InjectionDone,
    Fail,
    Reset,
}

impl AppState {
    pub fn apply(self, t: Transition) -> Self {
        match (self, t) {
            (AppState::Idle, Transition::StartRecording) => AppState::Recording,
            (AppState::Recording, Transition::StopRecording) => AppState::Transcribing,
            (AppState::Transcribing, Transition::TranscriptionDone) => AppState::Injecting,
            (AppState::Injecting, Transition::InjectionDone) => AppState::Idle,
            (_, Transition::Fail) => AppState::Error,
            (_, Transition::Reset) => AppState::Idle,
            (s, _) => s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path() {
        let s = AppState::Idle
            .apply(Transition::StartRecording)
            .apply(Transition::StopRecording)
            .apply(Transition::TranscriptionDone)
            .apply(Transition::InjectionDone);
        assert_eq!(s, AppState::Idle);
    }

    #[test]
    fn fail_from_any_state() {
        assert_eq!(AppState::Recording.apply(Transition::Fail), AppState::Error);
        assert_eq!(AppState::Transcribing.apply(Transition::Fail), AppState::Error);
    }

    #[test]
    fn invalid_transitions_are_noop() {
        assert_eq!(AppState::Idle.apply(Transition::StopRecording), AppState::Idle);
        assert_eq!(AppState::Idle.apply(Transition::InjectionDone), AppState::Idle);
    }

    #[test]
    fn reset_from_error() {
        assert_eq!(AppState::Error.apply(Transition::Reset), AppState::Idle);
    }
}
