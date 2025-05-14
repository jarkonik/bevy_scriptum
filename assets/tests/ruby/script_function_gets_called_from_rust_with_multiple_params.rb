STATE = {
	'a_value' => nil,
	'b_value' => nil
}

def test_func(a, b)
  STATE['a_value'] = a
  STATE['b_value'] = b
end
