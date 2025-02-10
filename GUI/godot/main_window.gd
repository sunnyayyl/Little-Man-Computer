extends Control

@onready var file_dialog:FileDialog=$FileDialog
@onready var file_menu_button:MenuButton=$VBoxContainer/HBoxContainer/MenuButton
@onready var filetree=$VBoxContainer/HSplitContainer/FileTree
func _ready() -> void:
	file_menu_button.get_popup().id_pressed.connect(id_pressed)
	file_dialog.dir_selected.connect(folder_selected)
func id_pressed(id:int):
	match id:
		0:
			file_dialog.show()
func folder_selected(path:String):
	filetree.watch(path)
