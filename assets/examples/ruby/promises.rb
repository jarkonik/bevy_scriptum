def async_fun
  async do
    a = get_player_name
    b = a
    puts '0'
    puts a.await
    puts '1'
    u = get_player_name
    puts b.await
    puts '2'
    z = get_player_name
    puts z
    puts z.await
    puts "end"
  end
end

async_fun.await
puts "after await"

quit
