# nugem

**nugem** is a 2D fighting game engine aiming for compatibility with [Mugen](https://en.wikipedia.org/wiki/Mugen_(game_engine)).

Currently, the functionalities are limited to rendering sprites and animations. You can change the displayed character and animation using directional inputs on the controller. 

Arguments:
* `--data  path/to/data/folder/` add a data folder (can be multiple). A data folder may contain subfolders for Mugen characters.
Others to be documented

### Keyboard mappings

 * alphanumerical keys mapping: WASD for direction, U/I/O for A/B/C buttons, J/K/L for X/Y/Z buttons, return (enter) for start, backspace for back
 * numpad keys mapping: directional arrows for direction, numpad 7/8/9 for A/B/C buttons, numpad 4/5/6 for X/Y/Z buttons, numpad enter for start, numpad comma for back

## Reference

### Mugen file compatibility

* [Elecbyte wiki on the Internet Archive](https://web.archive.org/web/20150613185024/http://elecbyte.com/wiki/index.php/Main_Page)
* [Resources from the official MUGEN builds](https://mugenarchive.com/forums/downloads.php?do=cat&id=39-mugen-builds)

### WGPU

 * [WGPU tutorial](https://sotrh.github.io/learn-wgpu/)
 * [Shaders: WGSL reference](https://www.w3.org/TR/WGSL/)
 * [WebGPU API reference](https://gpuweb.github.io/gpuweb/#enumdef-gpufiltermode)
