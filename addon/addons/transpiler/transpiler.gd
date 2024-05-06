@tool
extends EditorPlugin

var last_selected: TreeItem

func _enter_tree():
	var fs = EditorInterface.get_file_system_dock()
	var menu = fs.get_child(2) as PopupMenu
	var tree = fs.get_child(3).get_child(0) as Tree
	
	menu.id_pressed.connect(_pressed)
	tree.multi_selected.connect(_selected)
	resource_saved.connect(_on_resource_saved)

func _pressed(id: int):
	if id == 100:
		generate(get_path_from_item(last_selected))


func _selected(item: TreeItem, colom: int, selected: bool):
	if selected:
		last_selected = item

func _on_resource_saved(res):
	if res is GDScript:
		if res.source_code.begins_with("#[autogen]\n"):
			generate(res.resource_path)

func _process(delta: float) -> void:
	var fs = EditorInterface.get_file_system_dock()
	var menu = fs.get_child(2) as PopupMenu
	var tree = fs.get_child(3).get_child(0) as Tree
	#print(menu.item_count)
	if (
		menu.visible 
		and menu.item_count == 17
		and last_selected.get_text(0).ends_with(".gd")
		#and menu.get_item_text(0) == "Open"
	):
		menu.add_item("Translate To Rust", 100)
		menu.popup()
	


func get_path_from_item(item: TreeItem) -> String:
	var parent = item.get_parent()
	if parent != null:
		return (get_path_from_item(parent) + "/" 
		+ item.get_text(0)).replace("/res:///", "res://")
	else:
		return ""


func _exit_tree():
	var fs = EditorInterface.get_file_system_dock()
	var menu = fs.get_child(2) as PopupMenu
	var tree = fs.get_child(3).get_child(0) as Tree
	
	menu.id_pressed.disconnect(_pressed)
	tree.multi_selected.disconnect(_selected)
	resource_saved.disconnect(_on_resource_saved)


func generate(path: String):
	#print(path)
	var code = load(path) as GDScript
	var src = code.source_code
	
	var transpiler = GDScriptTranspiler.new()
	var rust_src = transpiler.transpile_to_rust(src)
	
	var rust_path = make_rust_file(path)
	#print(rust_path)
	var file = FileAccess.open(rust_path, FileAccess.WRITE_READ)
	file.store_string(rust_src)
	file.close()
	
	#OS.execute("cargo", [
	#	"clippy", 
	#	"--allow-dirty", 
	#	"--manifest-path", 
	#	"addons/transpiler/rust/Cargo.toml",
	#	"--fix"
	#])
	
	OS.execute("rustfmt", [rust_path.substr(6)])
	OS.execute("cargo", [
		"build", 
		"--manifest-path", 
		"addons/transpiler/rust/Cargo.toml",
		"--target-dir",
		"addons/transpiler/rust/target"
	])
	


func make_rust_file(path: String) -> String:
	return "res://addons/transpiler/rust/src/" + path.substr(6, path.length() - 8) + "rs"
