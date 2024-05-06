#[autogen]
class_name AutogenTest
extends Node


func _ready() -> void:
	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass

func test():
	for i in 100000:
		fib()

func fib() -> int:
	var n1 := 0
	var n2 := 1
	for i in range(0, 91):
		var n := n2
		n2 = n2 + n1
		n1 = n

	return n2
