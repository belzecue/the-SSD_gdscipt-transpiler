extends Node


func _ready():
	var gd = AutogenTest.new()
	var rust = AutogenTest_rs.new()
	
	var start_time = Time.get_ticks_msec()
	gd.test()
	var end_time = Time.get_ticks_msec()
	print("Time to run GDScript: ", end_time - start_time)
	
	start_time = Time.get_ticks_msec()
	rust.test()
	end_time = Time.get_ticks_msec()
	print("Time to run Rust: ", end_time - start_time)
	

