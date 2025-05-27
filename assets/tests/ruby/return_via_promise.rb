$state = {
  'x' => nil
}

def test_func
  rust_func.and_then do |x|
    $state['x'] = x
  end
end
