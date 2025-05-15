def test_func
  rust_func.and_then do |x|
    print('abc' + 5)
  end
end
