-- entity is a global variable that is set to the entity that is currently being processed,
-- it is automatically available in all scripts

-- get name of the entity
entity:get_name():and_then(function(name)
	print(name)
end)
