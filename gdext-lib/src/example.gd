class_name Test
extends Node

#func test(test_arg: int) -> int:
#    var test_var := 0
#    if test_arg == 5:
#        if true:
#            test_var = 1
#    
#    return test_var

func test():
	var i = 0
	while i < 5:
		print(i)
		i += 1