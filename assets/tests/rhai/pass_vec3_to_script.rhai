fn test_func(vec3) {
  if type_of(vec3) != "Vec3" { throw() }
  if vec3.x != 1.5 { throw() }
  if vec3.y != 2.5 { throw() }
  if vec3.z != -3.5 { throw() }
  mark_success();
}
