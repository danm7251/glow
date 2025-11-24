# TODO
## src/app/mod.rs
* [src/app/mod.rs:67](src/app/mod.rs#L67): Fix bug when pausing and using label to play
* [src/app/mod.rs:107](src/app/mod.rs#L107): filter songs by artist
* [src/app/mod.rs:120](src/app/mod.rs#L120): Centralise string conversion

## src/audio.rs
* [src/audio.rs:7](src/audio.rs#L7): Review code, add minimal Doc
* [src/audio.rs:29](src/audio.rs#L29): Change return type to anyhow::Result

## src/library/mod.rs
* [src/library/mod.rs:53](src/library/mod.rs#L53): Accept folders
* [src/library/mod.rs:75](src/library/mod.rs#L75): Refactor once Song -> Result<Song>, Consider what to do with the file copy
* [src/library/mod.rs:134](src/library/mod.rs#L134): Refactor once Song -> Result<Song>

## src/library/song.rs
* [src/library/song.rs:4](src/library/song.rs#L4): Write minimal documentation
* [src/library/song.rs:5](src/library/song.rs#L5): Prototype an enum and field to track the status of the file
* [src/library/song.rs:15](src/library/song.rs#L15): Convert return type to Result
* [src/library/song.rs:16](src/library/song.rs#L16): Review and improve function, for example the type of parameter path
