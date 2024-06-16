let state = #{
	a_value: (),
	b_value: ()
};

fn test_func(a, b) {
	state.a_value = a;
	state.b_value = b;
}
