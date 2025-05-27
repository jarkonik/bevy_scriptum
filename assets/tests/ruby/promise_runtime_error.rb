def test_func
  rust_func.and_then do |x|
    raise
  end
end
