$state = {
  'times_called' => 0
}

def test_func
  $state['times_called'] += 1
end
