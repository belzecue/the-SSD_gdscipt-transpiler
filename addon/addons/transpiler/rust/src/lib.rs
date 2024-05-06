use godot::prelude::*;
mod autogen_test;

struct GDScriptTranspiler;

#[gdextension]
unsafe impl ExtensionLibrary for GDScriptTranspiler {}
