// entity is a global variable that is set to the entity that is currently being processed,
// it is automatically available in all scripts

// get name of the entity using registered function
get_name(entity).then(|name| {
    print(name);
});

// Rhai also supports calling functions with the dot operator
entity.get_name().then(|name| {
    print(name);
})

