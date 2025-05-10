STATE = {
  "times_called" => 0
}

def test_func
  STATE["times_called"] += 1
end
