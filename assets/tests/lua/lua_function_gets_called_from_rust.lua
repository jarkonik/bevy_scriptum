local state = {
	times_called = 0
};

function Test_func()
	state.times_called = state.times_called + 1;
end
