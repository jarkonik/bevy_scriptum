let state = #{ 
	times_called: 0
};

fn test_func() {
	state.times_called += 1;
}
