let state = #{
	x: 0
};

fn test_func() {
	rust_func().then(|x| {
		state.x = x;
	})
}
