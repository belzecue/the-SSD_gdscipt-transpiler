@tool
extends EditorPlugin

#var last_selected: TreeItem
var logo = preload("res://addons/transpiler/rustacean-flat-happy.svg")

func _enter_tree():
	var fs = EditorInterface.get_file_system_dock()
	var menu = fs.get_child(2) as PopupMenu
	var tree = fs.get_child(3).get_child(0) as Tree
	
	menu.id_pressed.connect(_pressed)
	tree.deselect_all()
	resource_saved.connect(_on_resource_saved)

func _pressed(id: int):
	if id == 100:
		var fs = EditorInterface.get_file_system_dock()
		var tree = fs.get_child(3).get_child(0) as Tree
		var selected = tree.get_selected()
		generate(get_path_from_item(selected))

func _on_resource_saved(res):
	if res is GDScript:
		if res.source_code.begins_with("#[autogen]\n"):
			generate(res.resource_path)

func _process(delta: float) -> void:
	var fs = EditorInterface.get_file_system_dock()
	var menu = fs.get_child(2) as PopupMenu
	var tree = fs.get_child(3).get_child(0) as Tree
	#print(menu.item_count)Tree
	var selected = tree.get_selected()
	
	if (
		menu.visible 
		and menu.get_item_index(100) == -1
		and selected.get_text(0).ends_with(".gd")
	):
		#menu.add_item("Translate To Rust", 100)
		
		menu.add_icon_item(logo, "Translate To Rust", 100)
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
	#	"rust/Cargo.toml",
	#	"--fix"
	#])
	
	OS.execute("rustfmt", [rust_path.substr(6)])
	OS.execute("cargo", [
		"build", 
		"--manifest-path", 
		"rust/Cargo.toml",
		"--target-dir",
		"rust/target"
	])
	


func make_rust_file(path: String) -> String:
	return "res://rust/src/" + path.substr(6, path.length() - 8) + "rs"
