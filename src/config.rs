const FOLDER: &str = "test";
const ALLOW_NON_MP3: bool = false;

pub struct Config {
    folder: &'static str,
    allow_non_mp3: bool,
}

impl Config {
    pub fn dev_preset() -> Self {
        Self {
            folder: FOLDER,
            allow_non_mp3: ALLOW_NON_MP3,
        }
    }

    pub fn folder(&self) -> &str {
        self.folder
    }

    pub fn allow_non_mp3(&self) -> bool {
        self.allow_non_mp3
    }
}
