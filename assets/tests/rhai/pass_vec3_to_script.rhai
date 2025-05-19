fn test_func(vec3) {
  if type_of(vec3) != "tests::rhai_tests::BevyVec3" { throw() }
  // TODO: assert x,y,z
  mark_success();
}
