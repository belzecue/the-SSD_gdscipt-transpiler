class_name Test
extends Node

func fib() -> int:
	var n1 := 0
	var n2 := 1
	for i in range(0, 1000000):
		var n := n2
		n2 = n2 + n1
		n1 = n

	return n2