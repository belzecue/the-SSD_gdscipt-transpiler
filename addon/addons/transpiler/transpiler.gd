@tool
extends EditorPlugin

var last_selected: TreeItem

func _enter_tree():
	var fs = EditorInterface.get_file_system_dock()
	var menu = fs.get_child(2) as PopupMenu
	var tree = fs.get_child(3).get_child(0) as Tree
	
	menu.id_pressed.connect(pressed)
	tree.multi_selected.connect(selected)
	resource_saved.connect(_on_resource_saved)

func _on_resource_saved(res):
	if res is GDScript:
		if res.source_code.begins_with("#[autogen]\n"):
			generate(res.resource_path)

func generate(path: String):
	print("Generate ", path)

func _process(delta: float) -> void:
	var fs = EditorInterface.get_file_system_dock()
	var menu = fs.get_child(2) as PopupMenu
	var tree = fs.get_child(3).get_child(0) as Tree
	
	if (
		menu.visible 
		and menu.item_count == 19
		and last_selected.get_text(0).ends_with(".gd")
		#and menu.get_item_text(0) == "Open"
	):
		menu.add_item("Translate To Rust", 100)
		menu.popup()
	

func pressed(id: int):
	if id == 100:
		generate(get_path_from_item(last_selected))


func get_path_from_item(item: TreeItem) -> String:
	var parent = item.get_parent()
	if parent != null:
		return (get_path_from_item(parent) + "/" 
		+ item.get_text(0)).replace("/res:///", "res://")
	else:
		return ""

func selected(item: TreeItem, colom: int, selected: bool):
	if selected:
		last_selected = item

func _exit_tree():
	var fs = EditorInterface.get_file_system_dock()
	var menu = fs.get_child(2) as PopupMenu
	var tree = fs.get_child(3).get_child(0) as Tree
	
	menu.id_pressed.disconnect(pressed)
	tree.multi_selected.disconnect(selected)
