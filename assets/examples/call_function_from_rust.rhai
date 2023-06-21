let my_state = #{
    iterations: 0,
};

fn on_update() {
    my_state.iterations += 1;
    print("on_update called " + my_state.iterations + " times");

    if (my_state.iterations >= 10) {
        print("calling quit");
        quit();
    }
}
