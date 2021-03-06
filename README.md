# chip8-rs
[Chip-8](https://en.wikipedia.org/wiki/CHIP-8) emulator, written in rust, complete with keyboard and sound support. Some freely available games are included in `roms` directory.

<img src="assets/out.gif">

## Running

Requires SDL2 to be installed. On Ubuntu, run

```
sudo apt install libsdl2-dev
```

then

```
cargo run -- roms/TETRIS
```

## Resources

The following resources have been a huge help

- https://en.wikipedia.org/wiki/CHIP-8
- http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
- http://www.multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/


## License

Licensed under GPLv3. Sample games included in `roms` directory are written by other authors and are under other licenses.
