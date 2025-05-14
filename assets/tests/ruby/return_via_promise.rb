STATE = {
  'x' => nil
}

def test_func
  rust_func.and_then do |x|
    STATE['x'] = x
  end
end
