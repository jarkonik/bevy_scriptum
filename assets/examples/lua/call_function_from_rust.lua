local my_state = {
	iterations = 0,
}

function on_update()
	my_state.iterations = my_state.iterations + 1;
	print("on_update called " .. my_state.iterations .. " times")

	if my_state.iterations >= 10 then
		print("calling quit");
		quit();
	end
end
