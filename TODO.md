# REVIEW
## src\player.rs
* [src\player.rs:48](src\player.rs#L48): [LATER] Return type
# TODO
## src\app\mod.rs
* [src\app\mod.rs:52](src\app\mod.rs#L52): [SOON] Consider how the main loop can facilitate clean communication between modules
* [src\app\mod.rs:110](src\app\mod.rs#L110): [LATER] Need some kind of intermediate Vector to represent 'views' into the Library. Consider that the Library showcases data and the new layer will encapsulate actions
* [src\app\mod.rs:128](src\app\mod.rs#L128): [LATER] Filter tracklist by artist
* [src\app\mod.rs:142](src\app\mod.rs#L142): [LATER] Finalize the playback bar design
* [src\app\mod.rs:177](src\app\mod.rs#L177): [LATER] Style the seek bar

## src\library\mod.rs
* [src\library\mod.rs:68](src\library\mod.rs#L68): [LATER] Accept dropped folders

## src\library\song.rs
* [src\library\song.rs:8](src\library\song.rs#L8): [LATER] An enum should exist with rich information on the validity of the file. Possibly in Library.

## src\player.rs
* [src\player.rs:39](src\player.rs#L39): [SOON] Handle no audio
* [src\player.rs:163](src\player.rs#L163): [SOON] Automate tests with a commit hook and CI using github actions
