$state = {
  'called_with' => nil
}

def test_func(val)
  $state['called_with'] = val
end
