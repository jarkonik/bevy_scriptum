# Builtin variables

## entity

Current entity that the script is atteched to can be retrieved by calling `Bevy::Entity.current`.
It exposes `index` method that returns bevy entity index.
It is useful for accessing entity's components from scripts.
It can be used in the following way:
```ruby
puts("Current entity index: #{Bevy::Entity.current.index}")
```

`Bevy::Entity.current` variable is currently not available within promise callbacks.
