# FIXME
## src\app\mod.rs
* [src\app\mod.rs:168](src\app\mod.rs#L168): [LATER] Consider this side of the playbacks role in the AudioEngine::play_song bug

## src\audio.rs
* [src\audio.rs:57](src\audio.rs#L57): [LATER] Find a better solution to the issue of playing a new song while the engine is paused
# TODO
## src\app\mod.rs
* [src\app\mod.rs:60](src\app\mod.rs#L60): [SOON] Consider how the main loop can facilitate clean communication between modules
* [src\app\mod.rs:120](src\app\mod.rs#L120): [LATER] Need some kind of intermediate Vector to represent 'views' into the Library. Consider that the Library showcases data and the new layer will encapsulate actions
* [src\app\mod.rs:144](src\app\mod.rs#L144): [LATER] Filter tracklist by artist
* [src\app\mod.rs:158](src\app\mod.rs#L158): [LATER] Finalize the playback bar design
* [src\app\mod.rs:187](src\app\mod.rs#L187): [SOON] Consider another layer of abstraction
* [src\app\mod.rs:199](src\app\mod.rs#L199): [LATER] Current song should be dropped when song ends
* [src\app\mod.rs:200](src\app\mod.rs#L200): [LATER] Style the seek bar
* [src\app\mod.rs:201](src\app\mod.rs#L201): [SOON] Review and revise seek logic according to pending arch. review

## src\audio.rs
* [src\audio.rs:7](src\audio.rs#L7): [SOON] Review and refine code
* [src\audio.rs:8](src\audio.rs#L8): [SOON] Document module
* [src\audio.rs:9](src\audio.rs#L9): [LATER] Stop rodio printing when the OutputStream is dropped in developer builds
* [src\audio.rs:16](src\audio.rs#L16): [SOON] Encapsulate field
* [src\audio.rs:18](src\audio.rs#L18): [SOON] Consider removing this field and giving it to a playback controller
* [src\audio.rs:58](src\audio.rs#L58): [SOONEST] self.sink.stop()

## src\library\mod.rs
* [src\library\mod.rs:68](src\library\mod.rs#L68): [LATER] Accept dropped folders
* [src\library\mod.rs:175](src\library\mod.rs#L175): [SOON] Stop silent fails caused by ok()

## src\library\song.rs
* [src\library\song.rs:8](src\library\song.rs#L8): [LATER] An enum should exist with rich information on the validity of the file. Possibly in Library.

## src\main.rs
* [src\main.rs:6](src\main.rs#L6): [URGENT] A high level architecture review. Consider the benefits of separating playback state from GUI state. Consider how the AudioEngine, Library and GlowApp interact. It is acceptable for the GUI to poll the internals thats just the design paradigm that egui provides developers with.

## src\player.rs
* [src\player.rs:152](src\player.rs#L152): [SOON] Automate tests with a commit hook and CI using github actions
