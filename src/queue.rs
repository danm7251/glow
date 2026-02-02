pub struct Queue {
    played: Vec<usize>,
    upcoming: Vec<usize>,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            played: Vec::<usize>::new(),
            upcoming: Vec::<usize>::new(),
        }
    }

    pub fn from_playlist(songs: Vec<usize>) -> Self {
        Self {
            played: Vec::<usize>::new(),
            upcoming: songs,
        }
    }

    pub fn add(&mut self, id: usize) {
        self.upcoming.push(id);
    }

    pub fn next(&mut self) -> Option<usize> {
        self.upcoming.pop()
    }
}
