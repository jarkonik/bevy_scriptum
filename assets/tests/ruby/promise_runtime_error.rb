def test_func
  rust_func.and_then lambda { |x|
    print('abc' + 5)
  }
end
