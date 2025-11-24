# TODO
## src/app.rs
* [src/app.rs:89](src/app.rs#L89): Fix bug when pausing and using label to play
* [src/app.rs:131](src/app.rs#L131): filter songs by artist
* [src/app.rs:174](src/app.rs#L174): Refactor to use anyhow

## src/library.rs
* [src/library.rs:52](src/library.rs#L52): Accept folders
* [src/library.rs:74](src/library.rs#L74): Refactor once Song -> Result<Song>, Consider what to do with the file copy
* [src/library.rs:133](src/library.rs#L133): Refactor once Song -> Result<Song>

## src/song.rs
* [src/song.rs:4](src/song.rs#L4): Write minimal documentation
* [src/song.rs:5](src/song.rs#L5): Change all fields to private
* [src/song.rs:6](src/song.rs#L6): Consider renaming fields
* [src/song.rs:17](src/song.rs#L17): Convert return type to Result
* [src/song.rs:18](src/song.rs#L18): Review and improve function, for example the type of parameter path
