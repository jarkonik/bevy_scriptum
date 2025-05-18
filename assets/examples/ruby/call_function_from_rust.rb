$my_state = {
  iterations: 0,
}

def on_update
  $my_state[:iterations] += 1
  puts("on_update called #{$my_state[:iterations]} times")

  if $my_state[:iterations] >= 10
    print("calling quit");
    quit()
  end
end
