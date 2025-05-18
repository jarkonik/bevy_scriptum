def test_func(vec3)
  raise unless vec3.is_a?(Bevy::Vec3) # TODO: BevyScriptum::Vec3 and add example how to include it globally like Sinatra does
  raise unless vec3.x == 1.5
  raise unless vec3.y == 2.5
  raise unless vec3.z == -3.5
  mark_success
end
