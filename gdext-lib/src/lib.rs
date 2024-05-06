mod example;
mod transpiler;

use godot::prelude::*;
struct Example;

#[gdextension]
unsafe impl ExtensionLibrary for Example {}
