State = {
	x = nil
}

function test_func()
	rust_func():and_then(function(x)
		State.x = x
	end)
end
