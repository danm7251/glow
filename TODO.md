# REVIEW
## src\player.rs
* [src\player.rs:48](src\player.rs#L48): [LATER] Return type
# TODO
## src\app\mod.rs
* [src\app\mod.rs:55](src\app\mod.rs#L55): [SOON] Consider how the main loop can facilitate clean communication between modules
* [src\app\mod.rs:79](src\app\mod.rs#L79): [SOON] Force all buttons to be equal size
* [src\app\mod.rs:100](src\app\mod.rs#L100): [LATER] Add functionality
* [src\app\mod.rs:103](src\app\mod.rs#L103): [LATER] Add functionality
* [src\app\mod.rs:123](src\app\mod.rs#L123): [LATER] Add functionality
* [src\app\mod.rs:126](src\app\mod.rs#L126): [LATER] Add functionality
* [src\app\mod.rs:186](src\app\mod.rs#L186): [LATER] Need some kind of intermediate Vector to represent 'views' into the Library. Consider that the Library showcases data and the new layer will encapsulate actions
* [src\app\mod.rs:204](src\app\mod.rs#L204): [LATER] Filter tracklist by artist
* [src\app\mod.rs:218](src\app\mod.rs#L218): [LATER] Finalize the playback bar design
* [src\app\mod.rs:264](src\app\mod.rs#L264): [SOON] Format position to minutes and seconds (.custom_formatter())
* [src\app\mod.rs:265](src\app\mod.rs#L265): [LATER] Style the seek bar (.handle_shape())

## src\library\mod.rs
* [src\library\mod.rs:68](src\library\mod.rs#L68): [LATER] Accept dropped folders

## src\library\song.rs
* [src\library\song.rs:8](src\library\song.rs#L8): [LATER] An enum should exist with rich information on the validity of the file. Possibly in Library.

## src\player.rs
* [src\player.rs:39](src\player.rs#L39): [SOON] Handle no audio
* [src\player.rs:163](src\player.rs#L163): [SOON] Automate tests with a commit hook and CI using github actions
