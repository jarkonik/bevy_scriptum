def test_func
  rust_func.and_then lambda { |x|
    $state['x'] = x
  }
end
