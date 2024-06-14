# Builtin variables

## entity

A variable called `entity` is automatically available to all scripts - it represents bevy entity that the `Script` component is attached to.
It exposes `index` property that returns bevy entity index.
It is useful for accessing entity's components from scripts.
It can be used in the following way:
```lua
print("Current entity index: " .. entity.index)
```

`entity` variable is currently not available within promise callbacks.
