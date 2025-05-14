$state = {
  'a_value' => nil,
  'b_value' => nil
}

def test_func(a, b)
  $state['a_value'] = a
  $state['b_value'] = b
end
