use ohmers::{Reference, Collection};

model!(
    derive { Clone }
    User {
        uniques {
            name : String = "".into();
        };

        tracks : Collection<TimeTrack> = Collection::new();
    }
);

model!(
    derive { Clone }
    TimeTrack {
        indices { user : Reference<User> = Reference::new(); };

        start : u64 = 0;
        stop : u64 = 0;
    }
);

#[derive(Debug,RustcEncodable)]
pub struct TimeTrackView {
    id: usize,
    start: u64,
    stop: u64
}

impl TimeTrackView {
    pub fn from(track: &TimeTrack) -> TimeTrackView {
        TimeTrackView {
            id: track.id,
            start: track.start,
            stop: track.stop,
        }
    }
}
