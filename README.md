# Manic Miner clone using the Bevy game engine

This is just an experiment, but it's a reasonably somewhat functional, albeit with bugs version of Manic Miner written in Rust using the Bevy game engine.

I wrote this when I was early on learning rust, so it might have some rough corners. But it works!

`charset.bin` is the zx spectrum bitmap character set extracted from a spectrum 48k rom image using:

``dd skip=15616 count=768 if=48.rom of=assets/textures/charset.bin bs=1``

